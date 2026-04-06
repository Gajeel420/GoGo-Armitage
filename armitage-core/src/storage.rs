//! Storage layer for persisting data to PostgreSQL
//!
//! Provides database access and query operations for hosts, services,
//! sessions, credentials, and routes.

use crate::models::*;
use anyhow::Result;
use uuid::Uuid;

/// Main storage interface
pub trait Storage: Send + Sync {
    // Host operations
    async fn add_host(&self, host: Host) -> Result<()>;
    async fn update_host(&self, host: Host) -> Result<()>;
    async fn delete_host(&self, host_id: Uuid) -> Result<()>;
    async fn get_host(&self, host_id: Uuid) -> Result<Option<Host>>;
    async fn list_hosts(&self) -> Result<Vec<Host>>;

    // Service operations
    async fn add_service(&self, service: Service) -> Result<()>;
    async fn update_service(&self, service: Service) -> Result<()>;
    async fn delete_service(&self, service_id: Uuid) -> Result<()>;
    async fn get_service(&self, service_id: Uuid) -> Result<Option<Service>>;
    async fn list_services_for_host(&self, host_id: Uuid) -> Result<Vec<Service>>;

    // Session operations
    async fn add_session(&self, session: Session) -> Result<()>;
    async fn update_session(&self, session: Session) -> Result<()>;
    async fn close_session(&self, session_id: Uuid) -> Result<()>;
    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>>;
    async fn list_sessions(&self) -> Result<Vec<Session>>;

    // Credential operations
    async fn add_credential(&self, credential: Credential) -> Result<()>;
    async fn update_credential(&self, credential: Credential) -> Result<()>;
    async fn get_credential(&self, credential_id: Uuid) -> Result<Option<Credential>>;
    async fn list_credentials_for_host(&self, host_id: Uuid) -> Result<Vec<Credential>>;

    // Route operations
    async fn add_route(&self, route: Route) -> Result<()>;
    async fn remove_route(&self, route_id: Uuid) -> Result<()>;
    async fn list_routes(&self) -> Result<Vec<Route>>;

    // Loot operations
    async fn add_loot(&self, loot: Loot) -> Result<()>;
    async fn get_loot(&self, loot_id: Uuid) -> Result<Option<Loot>>;
    async fn list_loot_for_host(&self, host_id: Uuid) -> Result<Vec<Loot>>;
}

pub mod postgres;
