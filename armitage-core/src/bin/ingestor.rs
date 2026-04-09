//! Armitage Data Ingestor - Standalone data ingestion binary

use armitage_core::{DataIngestor, EventBroker};
use clap::Parser;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(name = "armitage-ingestor")]
#[command(about = "Armitage data ingestion service", long_about = None)]
struct Args {
    /// Metasploit RPC host
    #[arg(long, default_value = "127.0.0.1")]
    msf_host: String,

    /// Metasploit RPC port
    #[arg(long, default_value_t = 55553)]
    msf_port: u16,

    /// Metasploit RPC username
    #[arg(long, default_value = "msf")]
    msf_user: String,

    /// Metasploit RPC password
    #[arg(long, default_value = "password")]
    msf_pass: String,

    /// PostgreSQL database URL
    #[arg(long)]
    database_url: Option<String>,

    /// Event broker channel capacity
    #[arg(long, default_value_t = 1000)]
    event_capacity: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("armitage_core=info".parse()?),
        )
        .init();

    let args = Args::parse();
    info!("Starting Armitage Data Ingestor");
    info!("Connecting to Metasploit RPC at {}:{}", args.msf_host, args.msf_port);

    let event_broker = EventBroker::new(args.event_capacity);
    let _ingestor = DataIngestor::new(event_broker);

    // TODO: Implement main ingestion loop
    info!("Ingestor ready (stub implementation)");

    Ok(())
}
