//! Armitage API Server - REST and gRPC endpoints
//!
//! Runs both REST (HTTP) and gRPC servers for data access and team collaboration

use armitage_api::rest::{create_rest_app, AppState};
use armitage_core::{EventBroker, PostgresStorage};
use clap::Parser;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error};

#[derive(Parser, Debug)]
#[command(name = "armitage-api")]
#[command(about = "Armitage REST and gRPC API server")]
struct Args {
    /// Listen address for REST API
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Listen port for REST API
    #[arg(long, default_value_t = 8000)]
    port: u16,

    /// PostgreSQL connection string
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Event broker capacity
    #[arg(long, default_value_t = 1000)]
    broker_capacity: usize,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(format!("armitage_api={}", args.log_level).parse()?),
        )
        .init();

    info!("Starting Armitage API server v0.2.0");
    info!("Configuration:");
    info!("  REST API: {}:{}", args.host, args.port);
    info!("  Event broker capacity: {}", args.broker_capacity);

    // Initialize event broker
    let event_broker = EventBroker::new(args.broker_capacity);
    info!("Event broker initialized");

    // Initialize PostgreSQL storage (optional)
    let storage = if let Some(db_url) = &args.database_url {
        info!("Connecting to PostgreSQL: {}", db_url);
        match PostgresStorage::new(db_url).await {
            Ok(storage) => {
                if let Err(e) = storage.init().await {
                    error!("Failed to initialize database schema: {}", e);
                }
                Arc::new(storage)
            }
            Err(e) => {
                error!("Failed to connect to PostgreSQL: {}", e);
                return Err(e);
            }
        }
    } else {
        info!("Warning: DATABASE_URL not set, running without persistent storage");
        // Create a dummy storage for development
        Arc::new(PostgresStorage::new("postgresql://localhost").await?)
    };

    // Create application state
    let state = AppState {
        storage,
        event_broker: event_broker.clone(),
    };

    // Create REST app
    let rest_app = create_rest_app(state);

    // Start REST server
    let addr = format!("{}:{}", args.host, args.port);
    info!("Starting REST API server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;

    info!("REST API server listening on {}", listener.local_addr()?);
    info!("Endpoints:");
    info!("  Health: GET /api/v1/health");
    info!("  Hosts: GET /api/v1/hosts");
    info!("  Services: GET /api/v1/services");
    info!("  Sessions: GET /api/v1/sessions");
    info!("  Credentials: GET /api/v1/credentials");
    info!("  Teams: POST /api/v1/teams");
    info!("  Workspaces: POST /api/v1/workspaces");
    info!("  C2 Servers: GET /api/v1/c2-servers");
    info!("  Activity Log: GET /api/v1/activity-log");

    axum::serve(listener, rest_app)
        .await?;

    Ok(())
}
