use rustql_core::{
    executor::{Executor, ResolvedValue},
    schema::Schema,
    ast::Value,
};
use rustql_api::{start_server, auth::Auth};
use rustql_db::Database;
use std::collections::HashMap;
use std::sync::Arc;

async fn setup_executor(db: Database) -> Executor {
    let schema = Schema::new();
    let mut executor = Executor::new(schema);
    let auth = Arc::new(Auth::new("rustql_secret_key_2024"));

    // Hello resolver
    executor.add_resolver("Query.hello", |_, _| {
        Ok(ResolvedValue::String("Hello from RustQL! 🚀".to_string()))
    });

    // Users resolver
    let pool1 = Arc::clone(&db.pool);
    executor.add_resolver("Query.users", move |_, _| {
        let pool = Arc::clone(&pool1);
        let users = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query!("SELECT id, name, email FROM users")
                    .fetch_all(pool.as_ref())
                    .await
            })
        });
        match users {
            Ok(rows) => {
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
            Err(e) => Ok(ResolvedValue::String(format!("Error: {}", e)))
        }
    });

    // Register resolver
    let pool2 = Arc::clone(&db.pool);
    let auth2 = Arc::clone(&auth);
    executor.add_resolver("Mutation.register", move |field, _| {
        let pool = Arc::clone(&pool2);
        let auth = Arc::clone(&auth2);

        let name = field.arguments.iter()
            .find(|(k, _)| k == "name")
            .map(|(_, v)| match v {
                Value::String(s) => s.clone(),
                _ => "".to_string(),
            })
            .unwrap_or_default();

        let email = field.arguments.iter()
            .find(|(k, _)| k == "email")
            .map(|(_, v)| match v {
                Value::String(s) => s.clone(),
                _ => "".to_string(),
            })
            .unwrap_or_default();

        let password = field.arguments.iter()
            .find(|(k, _)| k == "password")
            .map(|(_, v)| match v {
                Value::String(s) => s.clone(),
                _ => "".to_string(),
            })
            .unwrap_or_default();

        let password_hash = Auth::hash_password(&password);

        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query!(
                    "INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3) RETURNING id, name, email",
                    name, email, password_hash
                )
                .fetch_one(pool.as_ref())
                .await
            })
        });

        match result {
            Ok(row) => {
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
    });

    // Login resolver
    let pool3 = Arc::clone(&db.pool);
    let auth3 = Arc::clone(&auth);
    executor.add_resolver("Mutation.login", move |field, _| {
        let pool = Arc::clone(&pool3);
        let auth = Arc::clone(&auth3);

        let email = field.arguments.iter()
            .find(|(k, _)| k == "email")
            .map(|(_, v)| match v {
                Value::String(s) => s.clone(),
                _ => "".to_string(),
            })
            .unwrap_or_default();

        let password = field.arguments.iter()
            .find(|(k, _)| k == "password")
            .map(|(_, v)| match v {
                Value::String(s) => s.clone(),
                _ => "".to_string(),
            })
            .unwrap_or_default();

        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query!(
                    "SELECT id, name, email, password_hash FROM users WHERE email = $1",
                    email
                )
                .fetch_optional(pool.as_ref())
                .await
            })
        });

        match result {
            Ok(Some(row)) => {
                let hash = row.password_hash.unwrap_or_default();
                if Auth::verify_password(&password, &hash) {
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
    });

    // Create User
    let pool4 = Arc::clone(&db.pool);
    executor.add_resolver("Mutation.createUser", move |field, _| {
        let pool = Arc::clone(&pool4);
        let name = field.arguments.iter()
            .find(|(k, _)| k == "name")
            .map(|(_, v)| match v {
                Value::String(s) => s.clone(),
                _ => "".to_string(),
            })
            .unwrap_or_default();
        let email = field.arguments.iter()
            .find(|(k, _)| k == "email")
            .map(|(_, v)| match v {
                Value::String(s) => s.clone(),
                _ => "".to_string(),
            })
            .unwrap_or_default();
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query!(
                    "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email",
                    name, email
                )
                .fetch_one(pool.as_ref())
                .await
            })
        });
        match result {
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
    });

    // Delete User
    let pool5 = Arc::clone(&db.pool);
    executor.add_resolver("Mutation.deleteUser", move |field, _| {
        let pool = Arc::clone(&pool5);
        let id: i32 = field.arguments.iter()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| match v {
                Value::Int(i) => *i as i32,
                _ => 0,
            })
            .unwrap_or(0);
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query!("DELETE FROM users WHERE id = $1", id)
                    .execute(pool.as_ref())
                    .await
            })
        });
        match result {
            Ok(_) => Ok(ResolvedValue::String("User deleted!".to_string())),
            Err(e) => Ok(ResolvedValue::String(format!("Error: {}", e)))
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