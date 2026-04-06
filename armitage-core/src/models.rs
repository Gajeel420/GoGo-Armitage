//! Core data models for Armitage
//!
//! Defines the fundamental data structures for hosts, services, sessions,
//! credentials, and routes discovered during cyber attack operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a discovered host/target on the network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Host {
    pub id: Uuid,
    pub address: String,
    pub ipv4: String,
    pub ipv6: Option<String>,
    pub mac_address: Option<String>,
    pub hostname: Option<String>,
    pub os_name: Option<String>,
    pub os_flavor: Option<String>,
    pub os_sp: Option<String>,
    pub os_lang: Option<String>,
    pub arch: Option<String>,
    pub purpose: Option<String>,
    pub info: Option<String>,
    pub comments: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a service discovered on a host
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Service {
    pub id: Uuid,
    pub host_id: Uuid,
    pub port: u16,
    pub proto: ServiceProto,
    pub state: ServiceState,
    pub name: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub extrainfo: Option<String>,
    pub method: Option<String>,
    pub conf: Option<i32>,
    pub info: Option<String>,
    pub comments: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Protocol type for services
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ServiceProto {
    Tcp,
    Udp,
}

impl std::fmt::Display for ServiceProto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceProto::Tcp => write!(f, "tcp"),
            ServiceProto::Udp => write!(f, "udp"),
        }
    }
}

/// Service state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ServiceState {
    Open,
    Closed,
    Filtered,
    Unknown,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceState::Open => write!(f, "open"),
            ServiceState::Closed => write!(f, "closed"),
            ServiceState::Filtered => write!(f, "filtered"),
            ServiceState::Unknown => write!(f, "unknown"),
        }
    }
}

/// Represents an active session (meterpreter or shell) on a host
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Session {
    pub id: Uuid,
    pub sid: u32,
    pub host_id: Uuid,
    pub session_type: SessionType,
    pub platform: String,
    pub user: Option<String>,
    pub via_exploit: Option<String>,
    pub via_payload: Option<String>,
    pub tunnel_peer: Option<String>,
    pub tunnel_local: Option<String>,
    pub target_host: Option<String>,
    pub target_port: Option<u16>,
    pub description: Option<String>,
    pub info: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Type of session
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    Meterpreter,
    Shell,
    VNC,
    RDP,
    Other,
}

impl std::fmt::Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionType::Meterpreter => write!(f, "meterpreter"),
            SessionType::Shell => write!(f, "shell"),
            SessionType::VNC => write!(f, "vnc"),
            SessionType::RDP => write!(f, "rdp"),
            SessionType::Other => write!(f, "other"),
        }
    }
}

/// Represents extracted credentials
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Credential {
    pub id: Uuid,
    pub host_id: Uuid,
    pub service_id: Option<Uuid>,
    pub origin_type: OriginType,
    pub private_type: PrivateType,
    pub private_data: String,
    pub public: Option<String>,
    pub realm: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ntlm_hash: Option<String>,
    pub lm_hash: Option<String>,
    pub ssh_key: Option<String>,
    pub jtr_format: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_type: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Origin type for credentials
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum OriginType {
    Exploit,
    Service,
    Session,
    Import,
}

impl std::fmt::Display for OriginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OriginType::Exploit => write!(f, "exploit"),
            OriginType::Service => write!(f, "service"),
            OriginType::Session => write!(f, "session"),
            OriginType::Import => write!(f, "import"),
        }
    }
}

/// Type of private data in credential
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum PrivateType {
    Password,
    Hash,
    SshKey,
}

impl std::fmt::Display for PrivateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivateType::Password => write!(f, "password"),
            PrivateType::Hash => write!(f, "hash"),
            PrivateType::SshKey => write!(f, "ssh_key"),
        }
    }
}

/// Represents a network route for pivoting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Route {
    pub id: Uuid,
    pub session_id: Uuid,
    pub subnet: String,
    pub netmask: String,
    pub gateway: String,
    pub metric: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Represents extracted loot/evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loot {
    pub id: Uuid,
    pub host_id: Uuid,
    pub session_id: Option<Uuid>,
    pub loot_type: String,
    pub data: Vec<u8>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Team Collaboration Models
// ============================================================================

/// Represents a team/organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a workspace within a team
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Workspace {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub visibility: WorkspaceVisibility,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Workspace visibility setting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceVisibility {
    /// Only owner can access
    Private,
    /// All team members can access
    Shared,
    /// Specific members based on permissions
    Restricted,
}

impl std::fmt::Display for WorkspaceVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceVisibility::Private => write!(f, "private"),
            WorkspaceVisibility::Shared => write!(f, "shared"),
            WorkspaceVisibility::Restricted => write!(f, "restricted"),
        }
    }
}

/// Represents a team member
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamMember {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
}

/// Role within a team
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum TeamRole {
    /// Can only view data
    Viewer = 0,
    /// Can modify data within workspace permissions
    Operator = 1,
    /// Can manage team members and workspaces
    Admin = 2,
    /// Full team control
    Owner = 3,
}

impl std::fmt::Display for TeamRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamRole::Viewer => write!(f, "viewer"),
            TeamRole::Operator => write!(f, "operator"),
            TeamRole::Admin => write!(f, "admin"),
            TeamRole::Owner => write!(f, "owner"),
        }
    }
}

/// Represents permissions for a user in a workspace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspacePermission {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: WorkspaceRole,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Uuid,
}

/// Role within a workspace
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceRole {
    /// Read-only access
    Viewer,
    /// Can run commands and modify data
    Operator,
    /// Can manage workspace settings and members
    Admin,
}

impl std::fmt::Display for WorkspaceRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceRole::Viewer => write!(f, "viewer"),
            WorkspaceRole::Operator => write!(f, "operator"),
            WorkspaceRole::Admin => write!(f, "admin"),
        }
    }
}

/// Represents a shared session across team members
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SharedSession {
    pub id: Uuid,
    pub session_id: Uuid,
    pub workspace_id: Uuid,
    pub shared_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Represents an activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub activity_type: ActivityType,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub action: String,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Types of activities that can be logged
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// Data discovery (host, service, credential)
    Discovery,
    /// Session opened or closed
    SessionEvent,
    /// Command executed
    CommandExecution,
    /// User/permission changes
    UserManagement,
    /// Workspace changes
    WorkspaceManagement,
    /// Settings or configuration changes
    Configuration,
    /// Audit or compliance event
    Audit,
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityType::Discovery => write!(f, "discovery"),
            ActivityType::SessionEvent => write!(f, "session_event"),
            ActivityType::CommandExecution => write!(f, "command_execution"),
            ActivityType::UserManagement => write!(f, "user_management"),
            ActivityType::WorkspaceManagement => write!(f, "workspace_management"),
            ActivityType::Configuration => write!(f, "configuration"),
            ActivityType::Audit => write!(f, "audit"),
        }
    }
}

/// Represents a C2 framework instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2Server {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub framework: C2Framework,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub connected: bool,
    pub last_connected: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// C2 framework types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum C2Framework {
    Metasploit,
    Sliver,
    CobaltStrike,
}

impl std::fmt::Display for C2Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            C2Framework::Metasploit => write!(f, "metasploit"),
            C2Framework::Sliver => write!(f, "sliver"),
            C2Framework::CobaltStrike => write!(f, "cobalt_strike"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_proto_display() {
        assert_eq!(ServiceProto::Tcp.to_string(), "tcp");
        assert_eq!(ServiceProto::Udp.to_string(), "udp");
    }

    #[test]
    fn test_service_state_display() {
        assert_eq!(ServiceState::Open.to_string(), "open");
        assert_eq!(ServiceState::Closed.to_string(), "closed");
    }
}
