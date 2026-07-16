pub mod auth;
pub mod rate_limit;
pub mod cache;

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
    safety::SchemaGuard,
    parser::Parser,
    executor::{Executor, ResolvedValue},
};
use crate::rate_limit::RateLimiter;
use crate::cache::Cache;

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
    pub cache: Option<Cache>,
    pub schema_guard: SchemaGuard,
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

    // Cache check
    let is_query = payload.query.trim().starts_with("query");
    if is_query {
        if let Some(cache) = &state.cache {
            let cache_key = format!("rustql:{}", payload.query);
            if let Some(cached) = cache.get(&cache_key).await {
                if let Ok(data) = serde_json::from_str(&cached) {
                    return (
                        StatusCode::OK,
                        Json(QueryResponse {
                            data: Some(data),
                            errors: None,
                        })
                    ).into_response();
                }
            }
        }
    }

    let mut parser = Parser::new(&payload.query);

    match parser.parse() {
        Ok(document) => {
            // Safety check
            if let Err(e) = state.schema_guard.validate(&document) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(QueryResponse {
                        data: None,
                        errors: Some(vec![format!("[SAFETY] {}", e)]),
                    })
                ).into_response();
            }

            match state.executor.execute(&document).await {
                Ok(data) => {
                    let mut json_map = serde_json::Map::new();
                    for (k, v) in &data {
                        json_map.insert(k.clone(), resolved_to_json(v));
                    }
                    let json_data = serde_json::Value::Object(json_map);

                    if is_query {
                        if let Some(cache) = &state.cache {
                            let cache_key = format!("rustql:{}", payload.query);
                            cache.set(&cache_key, &json_data.to_string()).await;
                        }
                    }

                    (
                        StatusCode::OK,
                        Json(QueryResponse {
                            data: Some(json_data),
                            errors: None,
                        })
                    ).into_response()
                }
                Err(e) => {
                    let status = match e.status_code() {
                        400 => StatusCode::BAD_REQUEST,
                        401 => StatusCode::UNAUTHORIZED,
                        404 => StatusCode::NOT_FOUND,
                        429 => StatusCode::TOO_MANY_REQUESTS,
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    (
                        status,
                        Json(QueryResponse {
                            data: None,
                            errors: Some(vec![
                                format!("[{}] {}", e.error_type(), e)
                            ]),
                        })
                    ).into_response()
                }
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(QueryResponse {
                data: None,
                errors: Some(vec![
                    format!("[PARSE_ERROR] {}", e)
                ]),
            })
        ).into_response(),
    }
}

pub fn create_app(executor: Executor) -> Router {
    let cache = Cache::new("redis://127.0.0.1/", 60).ok();

    let state = Arc::new(AppState {
        executor,
        jwt_secret: "rustql_secret_key_2024".to_string(),
        rate_limiter: RateLimiter::new(100, 60),
        cache,
        schema_guard: SchemaGuard::default(),
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
    println!("🔒 Rate Limit : 100 requests/minute per IP");
    println!("⚡ Cache      : Redis (60s TTL)");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    ).await.unwrap();
}