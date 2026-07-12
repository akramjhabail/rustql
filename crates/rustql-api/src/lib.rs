pub mod auth;
use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use rustql_core::{
    parser::Parser,
    executor::{Executor, ResolvedValue},
};

#[derive(Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub data: Option<serde_json::Value>,
    pub errors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct AppState {
    pub executor: Executor,
    pub jwt_secret: String,
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
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    let mut parser = Parser::new(&payload.query);

    match parser.parse() {
        Ok(document) => {
            match state.executor.execute(&document) {
                Ok(data) => {
                    let mut json_map = serde_json::Map::new();
                    for (k, v) in &data {
                        json_map.insert(k.clone(), resolved_to_json(v));
                    }
                    Json(QueryResponse {
                        data: Some(serde_json::Value::Object(json_map)),
                        errors: None,
                    })
                }
                Err(e) => Json(QueryResponse {
                    data: None,
                    errors: Some(vec![e.to_string()]),
                }),
            }
        }
        Err(e) => Json(QueryResponse {
            data: None,
            errors: Some(vec![e.to_string()]),
        }),
    }
}

pub fn create_app(executor: Executor) -> Router {
    let state = Arc::new(AppState {
        executor,
        jwt_secret: "rustql_secret_key_2024".to_string(),
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
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}