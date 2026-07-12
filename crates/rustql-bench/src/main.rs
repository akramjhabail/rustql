use chrono::Utc;
use colored::*;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

const SERVER_URL: &str = "http://localhost:4000/rustql";
const ITERATIONS: u32 = 100;

#[derive(Debug)]
struct BenchResult {
    name: String,
    total_ms: f64,
    avg_ms: f64,
    min_ms: f64,
    max_ms: f64,
    requests_per_sec: f64,
}

async fn bench_query(client: &Client, name: &str, query: &str) -> BenchResult {
    let mut times = Vec::new();

    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ = client
            .post(SERVER_URL)
            .json(&json!({"query": query}))
            .send()
            .await;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        times.push(elapsed);
    }

    let total_ms: f64 = times.iter().sum();
    let avg_ms = total_ms / ITERATIONS as f64;
    let min_ms = times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_ms = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let requests_per_sec = 1000.0 / avg_ms;

    BenchResult {
        name: name.to_string(),
        total_ms,
        avg_ms,
        min_ms,
        max_ms,
        requests_per_sec,
    }
}

fn print_result(result: &BenchResult) {
    println!("\n{}", "─".repeat(50).cyan());
    println!("📊 {}", result.name.yellow().bold());
    println!("{}", "─".repeat(50).cyan());
    println!("  ⚡ Avg Response : {:.2}ms", result.avg_ms);
    println!("  🔥 Min Response : {:.2}ms", result.min_ms);
    println!("  🐢 Max Response : {:.2}ms", result.max_ms);
    println!("  🚀 Req/sec      : {:.0}", result.requests_per_sec);
    println!("  📈 Total Time   : {:.2}ms", result.total_ms);
}

#[tokio::main]
async fn main() {
    println!("{}", "╔══════════════════════════════════════╗".green());
    println!("{}", "║      RustQL Benchmark Suite          ║".green());
    println!("{}", "╚══════════════════════════════════════╝".green());
    println!("🕐 Started at: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
    println!("🔄 Iterations: {}", ITERATIONS);

    let client = Client::new();

    // Test 1 — Simple Query
    println!("\n{}", "Running benchmarks...".yellow());
    
    let r1 = bench_query(
        &client,
        "Simple Hello Query",
        "query { hello }"
    ).await;
    print_result(&r1);

    // Test 2 — Users Query
    let r2 = bench_query(
        &client,
        "Users List Query",
        "query { users { id name email } }"
    ).await;
    print_result(&r2);

    // Test 3 — Mutation
    let r3 = bench_query(
        &client,
        "Create User Mutation",
        "mutation { createUser(name: \"BenchUser\", email: \"bench@test.com\") { id name } }"
    ).await;
    print_result(&r3);

    // Summary
    println!("\n{}", "═".repeat(50).green());
    println!("{}", "📊 BENCHMARK SUMMARY".green().bold());
    println!("{}", "═".repeat(50).green());
    
    let avg_rps = (r1.requests_per_sec + r2.requests_per_sec + r3.requests_per_sec) / 3.0;
    
    println!("  🏆 Best Query    : {:.2}ms ({})", r1.min_ms, r1.name);
    println!("  ⚡ Avg Req/sec   : {:.0}", avg_rps);
    println!("  🦀 Powered by    : Rust");
    
    if avg_rps > 1000.0 {
        println!("\n  {}", "🔥 BLAZING FAST! 1000+ requests/sec!".yellow().bold());
    } else if avg_rps > 500.0 {
        println!("\n  {}", "⚡ VERY FAST! 500+ requests/sec!".cyan().bold());
    } else {
        println!("\n  {}", "✅ Good performance!".green());
    }

    println!("\n{}", "Benchmark complete! 🦀".green().bold());
}