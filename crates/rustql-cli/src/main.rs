use rustql_core::{
    executor::{Executor, ResolvedValue},
    schema::Schema,
    ast::Value,
};
use rustql_api::start_server;
use rustql_db::Database;
use std::collections::HashMap;
use std::sync::Arc;

async fn setup_executor(db: Database) -> Executor {
    let schema = Schema::new();
    let mut executor = Executor::new(schema);

    // Hello resolver
    executor.add_resolver("Query.hello", |_, _| async {
        Ok(ResolvedValue::String("Hello from RustQL! 🚀".to_string()))
    });

    // Users resolver
    let pool1 = Arc::clone(&db.pool);
    executor.add_resolver("Query.users", move |_, _| {
        let pool = Arc::clone(&pool1);
        async move {
            let rows = sqlx::query!("SELECT id, name, email FROM users")
                .fetch_all(pool.as_ref())
                .await
                .unwrap_or_default();

            let user_list = rows.iter().map(|row| {
                let mut user = HashMap::new();
                user.insert("id".to_string(),
                    ResolvedValue::String(row.id.to_string()));
                user.insert("name".to_string(),
                    ResolvedValue::String(row.name.clone()));
                user.insert("email".to_string(),
                    ResolvedValue::String(row.email.clone()));
                ResolvedValue::Object(user)
            }).collect();

            Ok(ResolvedValue::List(user_list))
        }
    });

    // Register resolver
    let pool2 = Arc::clone(&db.pool);
    executor.add_resolver("Mutation.register", move |field, _| {
        let pool = Arc::clone(&pool2);
        let args = field.clone();
        async move {
            let name = args.iter()
                .find(|(k, _)| k == "name")
                .map(|(_, v)| match v {
                    Value::String(s) => s.clone(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();

            let email = args.iter()
                .find(|(k, _)| k == "email")
                .map(|(_, v)| match v {
                    Value::String(s) => s.clone(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();

            let password = args.iter()
                .find(|(k, _)| k == "password")
                .map(|(_, v)| match v {
                    Value::String(s) => s.clone(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();

            let password_hash = rustql_api::auth::Auth::hash_password(&password);

            match sqlx::query!(
                "INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3) RETURNING id, name, email",
                name, email, password_hash
            )
            .fetch_one(pool.as_ref())
            .await {
                Ok(row) => {
                    let auth = rustql_api::auth::Auth::new("rustql_secret_key_2024");
                    let token = auth.generate_token(&row.id.to_string(), &row.email);
                    let mut response = HashMap::new();
                    response.insert("id".to_string(),
                        ResolvedValue::String(row.id.to_string()));
                    response.insert("name".to_string(),
                        ResolvedValue::String(row.name.clone()));
                    response.insert("email".to_string(),
                        ResolvedValue::String(row.email.clone()));
                    response.insert("token".to_string(),
                        ResolvedValue::String(token));
                    Ok(ResolvedValue::Object(response))
                }
                Err(e) => Ok(ResolvedValue::String(format!("Error: {}", e)))
            }
        }
    });

    // Login resolver
    let pool3 = Arc::clone(&db.pool);
    executor.add_resolver("Mutation.login", move |field, _| {
        let pool = Arc::clone(&pool3);
        let args = field.clone();
        async move {
            let email = args.iter()
                .find(|(k, _)| k == "email")
                .map(|(_, v)| match v {
                    Value::String(s) => s.clone(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();

            let password = args.iter()
                .find(|(k, _)| k == "password")
                .map(|(_, v)| match v {
                    Value::String(s) => s.clone(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();

            match sqlx::query!(
                "SELECT id, name, email, password_hash FROM users WHERE email = $1",
                email
            )
            .fetch_optional(pool.as_ref())
            .await {
                Ok(Some(row)) => {
                    let hash = row.password_hash.unwrap_or_default();
                    if rustql_api::auth::Auth::verify_password(&password, &hash) {
                        let auth = rustql_api::auth::Auth::new("rustql_secret_key_2024");
                        let token = auth.generate_token(&row.id.to_string(), &row.email);
                        let mut response = HashMap::new();
                        response.insert("token".to_string(),
                            ResolvedValue::String(token));
                        response.insert("email".to_string(),
                            ResolvedValue::String(row.email.clone()));
                        Ok(ResolvedValue::Object(response))
                    } else {
                        Ok(ResolvedValue::String("Invalid password!".to_string()))
                    }
                }
                Ok(None) => Ok(ResolvedValue::String("User not found!".to_string())),
                Err(e) => Ok(ResolvedValue::String(format!("Error: {}", e)))
            }
        }
    });

    // Create User
    let pool4 = Arc::clone(&db.pool);
    executor.add_resolver("Mutation.createUser", move |field, _| {
        let pool = Arc::clone(&pool4);
        let args = field.clone();
        async move {
            let name = args.iter()
                .find(|(k, _)| k == "name")
                .map(|(_, v)| match v {
                    Value::String(s) => s.clone(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();

            let email = args.iter()
                .find(|(k, _)| k == "email")
                .map(|(_, v)| match v {
                    Value::String(s) => s.clone(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();

            match sqlx::query!(
                "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email",
                name, email
            )
            .fetch_one(pool.as_ref())
            .await {
                Ok(row) => {
                    let mut user = HashMap::new();
                    user.insert("id".to_string(),
                        ResolvedValue::String(row.id.to_string()));
                    user.insert("name".to_string(),
                        ResolvedValue::String(row.name.clone()));
                    user.insert("email".to_string(),
                        ResolvedValue::String(row.email.clone()));
                    Ok(ResolvedValue::Object(user))
                }
                Err(e) => Ok(ResolvedValue::String(format!("Error: {}", e)))
            }
        }
    });

    // Delete User
    let pool5 = Arc::clone(&db.pool);
    executor.add_resolver("Mutation.deleteUser", move |field, _| {
        let pool = Arc::clone(&pool5);
        let args = field.clone();
        async move {
            let id: i32 = args.iter()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| match v {
                    Value::Int(i) => *i as i32,
                    _ => 0,
                })
                .unwrap_or(0);

            match sqlx::query!("DELETE FROM users WHERE id = $1", id)
                .execute(pool.as_ref())
                .await {
                Ok(_) => Ok(ResolvedValue::String("User deleted!".to_string())),
                Err(e) => Ok(ResolvedValue::String(format!("Error: {}", e)))
            }
        }
    });

    executor
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!("╔════════════════════════════╗");
    println!("║     RustQL Server v0.1     ║");
    println!("╚════════════════════════════╝");

    let db_url = "postgres://rustql:rustql123@localhost/rustqldb";

    print!("🔌 Connecting to database... ");
    match Database::connect(db_url).await {
        Ok(db) => {
            println!("✅ Connected!");
            let executor = setup_executor(db).await;
            start_server(executor, 4000).await;
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
            std::process::exit(1);
        }
    }
}