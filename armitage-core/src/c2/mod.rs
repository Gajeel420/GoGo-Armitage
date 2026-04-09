//! C2 Framework Client Abstraction
//!
//! Provides a unified interface for different C2 frameworks (Metasploit, Sliver, etc.)

use crate::models::{Host, Service, Session, Credential};
use anyhow::Result;
use async_trait::async_trait;

/// Trait for C2 framework clients
#[async_trait]
pub trait C2Client: Send + Sync {
    /// Connect to the C2 server
    async fn connect(&self) -> Result<()>;

    /// Disconnect from the C2 server
    async fn disconnect(&self) -> Result<()>;

    /// Check if currently connected
    async fn is_connected(&self) -> Result<bool>;

    /// List all hosts from the C2 database
    async fn list_hosts(&self) -> Result<Vec<Host>>;

    /// List services for a specific host
    async fn list_services_for_host(&self, host_id: &str) -> Result<Vec<Service>>;

    /// List all active sessions
    async fn list_sessions(&self) -> Result<Vec<Session>>;

    /// Get session details
    async fn get_session(&self, session_id: u32) -> Result<Option<Session>>;

    /// Execute a command in a session
    async fn execute_command(&self, session_id: u32, command: &str) -> Result<String>;

    /// List extracted credentials
    async fn list_credentials(&self) -> Result<Vec<Credential>>;

    /// Get framework info/version
    async fn get_framework_info(&self) -> Result<FrameworkInfo>;
}

/// Information about the C2 framework
#[derive(Debug, Clone)]
pub struct FrameworkInfo {
    pub framework_name: String,
    pub version: String,
    pub connected: bool,
    pub database_size: Option<u64>,
}

pub mod metasploit;
pub mod sliver;

pub use metasploit::MetasploitClient;
pub use sliver::SliverClient;
