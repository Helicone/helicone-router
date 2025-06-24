use std::time::Duration;

use clap::{Parser, Subcommand};
use futures::StreamExt;
use rand::Rng;
use tokio::{
    signal,
    time::{Instant, sleep},
};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    subcommand_required = false,
    arg_required_else_help = false,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Send a single test request (default when no subcommand is provided)
    #[command(name = "test-request")]
    TestRequest {
        /// Stream the response
        #[arg(short, long, default_value_t = false)]
        stream: bool,

        /// Include the Authorization header
        #[arg(short = 'a', long = "auth", default_value_t = false)]
        auth: bool,

        /// Model identifier to use
        #[arg(
            short = 'm',
            long = "model",
            default_value = "openai/gpt-4o-mini"
        )]
        model: String,
    },

    /// Continuously send requests until interrupted (load testing)
    #[command(name = "load-test")]
    LoadTest {
        /// Stream the responses
        #[arg(short, long, default_value_t = false)]
        stream: bool,

        /// Include the Authorization header
        #[arg(short = 'a', long = "auth", default_value_t = false)]
        auth: bool,

        /// Model identifier to use
        #[arg(
            short = 'm',
            long = "model",
            default_value = "openai/gpt-4o-mini"
        )]
        model: String,
    },
}

impl Default for Commands {
    fn default() -> Self {
        Self::TestRequest {
            stream: false,
            auth: false,
            model: "openai/gpt-4o-mini".to_string(),
        }
    }
}

pub async fn test(
    print_response: bool,
    is_stream: bool,
    send_auth: bool,
    model: &str,
) {
    let openai_request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant that can answer
    questions and help with tasks."
            }
        ],
        "max_tokens": 400,
        "stream": is_stream
    });

    let openai_request: async_openai::types::CreateChatCompletionRequest =
        serde_json::from_value(openai_request_body).unwrap();

    let bytes = serde_json::to_vec(&openai_request).unwrap();

    let mut request_builder = reqwest::Client::new()
        .post("http://localhost:8080/ai/v1/chat/completions")
        .header("Content-Type", "application/json");

    if send_auth {
        if let Ok(helicone_api_key) =
            std::env::var("HELICONE_CONTROL_PLANE_API_KEY")
        {
            request_builder =
                request_builder.header("authorization", helicone_api_key);
        } else {
            eprintln!(
                "Warning: HELICONE_CONTROL_PLANE_API_KEY not set, skipping \
                 Authorization header."
            );
        }
    }

    let response = request_builder.body(bytes).send().await.unwrap();
    println!("Status: {}", response.status());
    let trace_id = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    println!("Trace ID: {}", trace_id);
    if print_response {
        if is_stream {
            let mut body_stream = response.bytes_stream();
            while let Some(Ok(chunk)) = body_stream.next().await {
                let json = serde_json::from_slice::<serde_json::Value>(&chunk)
                    .unwrap();
                let pretty_json = serde_json::to_string_pretty(&json).unwrap();
                println!("Chunk: {}", pretty_json);
            }
        } else {
            let response_bytes =
                response.json::<serde_json::Value>().await.unwrap();
            println!("Response: {}", response_bytes);
        }
    }
}

async fn run_forever_loop(is_stream: bool, send_auth: bool, model: String) {
    let mut rng = rand::thread_rng();
    let mut request_count = 0u64;
    let start_time = Instant::now();

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                let elapsed = start_time.elapsed();
                let rps = request_count as f64 / elapsed.as_secs_f64();
                println!("\nShutdown signal received!");
                println!("Total requests: {}", request_count);
                println!("Total time: {:.2}s", elapsed.as_secs_f64());
                println!("Average RPS: {:.2}", rps);
                break;
            }
            _ = async {
                test(false, is_stream, send_auth, &model).await;
                request_count += 1;

                if request_count % 100 == 0 {
                    let elapsed = start_time.elapsed();
                    let current_rps = request_count as f64 / elapsed.as_secs_f64();
                    println!("Requests sent: {}, Current RPS: {:.2}", request_count, current_rps);
                }

                let delay_ms = rng.gen_range(0..=2);
                sleep(Duration::from_millis(delay_ms)).await;
            } => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    dotenvy::dotenv().ok();

    match cli.command.unwrap_or_default() {
        Commands::TestRequest {
            stream,
            auth,
            model,
        } => {
            println!("Starting single test...");
            test(true, stream, auth, &model).await;
            println!("Test completed successfully!");
        }
        Commands::LoadTest {
            stream,
            auth,
            model,
        } => {
            println!("Starting load test - press Ctrl+C to stop...");
            run_forever_loop(stream, auth, model).await;
        }
    }
}
