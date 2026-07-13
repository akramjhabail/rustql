use rustql_core::executor::{Executor, ResolvedValue};
use rustql_core::schema::Schema;
use rustql_api::start_server;

#[tokio::main]
async fn main() {
    let schema = Schema::new();
    let mut executor = Executor::new(schema);

    executor.add_resolver("Query.hello", |_, _| async {
        Ok(ResolvedValue::String("Hello from RustQL!".to_string()))
    });

    start_server(executor, 4000).await;
}
