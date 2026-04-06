//! Armitage API Server - REST and gRPC endpoints

use armitage_api::create_rest_app;
use clap::Parser;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "armitage-api")]
#[command(about = "Armitage REST API server")]
struct Args {
    /// Listen address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Listen port
    #[arg(long, default_value_t = 8000)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("armitage_api=info".parse()?),
        )
        .init();

    let args = Args::parse();
    let addr = format!("{}:{}", args.host, args.port);

    info!("Starting API server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    let app = create_rest_app();

    axum::serve(listener, app)
        .await?;

    Ok(())
}
