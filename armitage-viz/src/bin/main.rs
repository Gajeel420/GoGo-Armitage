//! Armitage Visualization Service

use armitage_viz::NetworkGraph;
use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "armitage-viz")]
#[command(about = "Armitage visualization service")]
struct Args {
    /// Server listen address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server listen port
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    info!("Starting visualization service at {}:{}", args.host, args.port);

    let _graph = NetworkGraph::new();

    // TODO: Implement graph update handlers
    info!("Visualization service ready (stub implementation)");

    Ok(())
}
