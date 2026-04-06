//! Armitage Core - Real-time data analysis engine for Metasploit Framework
//!
//! This library provides the core functionality for ingesting, analyzing, and streaming
//! cyber attack management data from various sources including Metasploit RPC,
//! PostgreSQL, and live meterpreter/shell sessions.

pub mod models;
pub mod events;
pub mod storage;
pub mod ingestor;
pub mod analyzer;
pub mod msf;

pub use models::{
    Credential, Host, Loot, OriginType, PrivateType, Route, Service, ServiceProto, ServiceState,
    Session, SessionType,
};
pub use events::{Event, EventBroker};
pub use storage::Storage;
pub use ingestor::DataIngestor;
pub use analyzer::DataAnalyzer;
pub use msf::MsfClient;

// Re-export common types
pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;
