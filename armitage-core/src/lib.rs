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
pub mod c2;

pub use models::{
    ActivityLog, ActivityType, C2Framework, C2Server, Credential, Host, Loot, OriginType,
    PrivateType, Route, Service, ServiceProto, ServiceState, Session, SessionType, Team,
    TeamMember, TeamRole, Workspace, WorkspacePermission, WorkspaceRole, WorkspaceVisibility,
    SharedSession,
};
pub use events::{Event, EventBroker};
pub use storage::Storage;
pub use ingestor::DataIngestor;
pub use analyzer::DataAnalyzer;
pub use msf::MsfClient;
pub use c2::{C2Client, MetasploitClient, SliverClient, FrameworkInfo};

// Re-export common types
pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;
