pub mod auth;
pub mod rate_limit;

use axum::{
    routing::post,
    Router,
    Json,
    extract::{State, ConnectInfo},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use rustql_core::{
    parser::Parser,
    executor::{Executor, ResolvedValue},
};
use crate::rate_limit::RateLimiter;

#[derive(Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub data: Option<serde_json::Value>,
    pub errors: Option<Vec<String>>,
}

pub struct AppState {
    pub executor: Executor,
    pub jwt_secret: String,
    pub rate_limiter: RateLimiter,
}

fn resolved_to_json(value: &ResolvedValue) -> serde_json::Value {
    match value {
        ResolvedValue::String(s) => serde_json::Value::String(s.clone()),
        ResolvedValue::Int(i) => serde_json::json!(i),
        ResolvedValue::Float(f) => serde_json::json!(f),
        ResolvedValue::Bool(b) => serde_json::Value::Bool(*b),
        ResolvedValue::Null => serde_json::Value::Null,
        ResolvedValue::List(list) => {
            serde_json::Value::Array(
                list.iter().map(resolved_to_json).collect()
            )
        }
        ResolvedValue::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (k, v) in obj {
                map.insert(k.clone(), resolved_to_json(v));
            }
            serde_json::Value::Object(map)
        }
    }
}

async fn handle_query(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<QueryRequest>,
) -> impl IntoResponse {
    let ip = addr.ip().to_string();

    // Rate limit check
    if !state.rate_limiter.check(&ip).await {
        let remaining = state.rate_limiter.remaining(&ip).await;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(QueryResponse {
                data: None,
                errors: Some(vec![
                    format!("Rate limit exceeded! {} requests remaining.", remaining)
                ]),
            })
        ).into_response();
    }

    let mut parser = Parser::new(&payload.query);

    match parser.parse() {
        Ok(document) => {
            match state.executor.execute(&document).await {
                Ok(data) => {
                    let mut json_map = serde_json::Map::new();
                    for (k, v) in &data {
                        json_map.insert(k.clone(), resolved_to_json(v));
                    }
                    (
                        StatusCode::OK,
                        Json(QueryResponse {
                            data: Some(serde_json::Value::Object(json_map)),
                            errors: None,
                        })
                    ).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(QueryResponse {
                        data: None,
                        errors: Some(vec![e.to_string()]),
                    })
                ).into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(QueryResponse {
                data: None,
                errors: Some(vec![e.to_string()]),
            })
        ).into_response(),
    }
}

pub fn create_app(executor: Executor) -> Router {
    let state = Arc::new(AppState {
        executor,
        jwt_secret: "rustql_secret_key_2024".to_string(),
        rate_limiter: RateLimiter::new(100, 60), // 100 requests per minute
    });

    Router::new()
        .route("/rustql", post(handle_query))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn start_server(executor: Executor, port: u16) {
    let app = create_app(executor);
    let addr = format!("0.0.0.0:{}", port);
    println!("🚀 RustQL Server running on http://{}", addr);
    println!("🔒 Rate Limit: 100 requests/minute per IP");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    ).await.unwrap();
}