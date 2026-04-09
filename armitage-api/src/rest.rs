//! REST API implementation with data access and team management endpoints
//!
//! Provides HTTP endpoints for querying hosts, services, sessions, credentials,
//! and managing team collaboration features.

use armitage_core::{EventBroker, PostgresStorage, Host, Service, Session, Credential, Team, Workspace};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, error};
use uuid::Uuid;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<PostgresStorage>,
    pub event_broker: EventBroker,
}

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

/// Query parameters for list endpoints
#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Host list response
#[derive(Debug, Serialize)]
pub struct HostListResponse {
    pub hosts: Vec<Host>,
    pub total: usize,
}

/// Service list response
#[derive(Debug, Serialize)]
pub struct ServiceListResponse {
    pub services: Vec<Service>,
    pub total: usize,
}

/// Session list response
#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<Session>,
    pub total: usize,
}

/// Credential list response
#[derive(Debug, Serialize)]
pub struct CredentialListResponse {
    pub credentials: Vec<Credential>,
    pub total: usize,
}

/// Team management request
#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Workspace management request
#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub visibility: String,
}

/// Permission update request
#[derive(Debug, Deserialize)]
pub struct UpdatePermissionRequest {
    pub user_id: Uuid,
    pub role: String,
}

/// Create REST API router with all endpoints
pub fn create_rest_app(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/api/v1/health", get(health_check))

        // Data access endpoints
        .route("/api/v1/hosts", get(list_hosts))
        .route("/api/v1/hosts/:id", get(get_host))
        .route("/api/v1/services", get(list_services))
        .route("/api/v1/services/:id", get(get_service))
        .route("/api/v1/sessions", get(list_sessions))
        .route("/api/v1/sessions/:id", get(get_session))
        .route("/api/v1/credentials", get(list_credentials))
        .route("/api/v1/credentials/:id", get(get_credential))

        // Team management endpoints
        .route("/api/v1/teams", post(create_team))
        .route("/api/v1/teams/:id", get(get_team))
        .route("/api/v1/teams/:id/members", get(list_team_members))
        .route("/api/v1/teams/:id/members", post(add_team_member))
        .route("/api/v1/teams/:id/members/:user_id", delete(remove_team_member))

        // Workspace management endpoints
        .route("/api/v1/workspaces", post(create_workspace))
        .route("/api/v1/workspaces/:id", get(get_workspace))
        .route("/api/v1/workspaces/:id/permissions", get(list_workspace_permissions))
        .route("/api/v1/workspaces/:id/permissions", put(update_workspace_permission))

        // C2 framework management endpoints
        .route("/api/v1/c2-servers", get(list_c2_servers))
        .route("/api/v1/c2-servers", post(register_c2_server))
        .route("/api/v1/c2-servers/:id/connect", post(connect_c2_server))

        // Activity log endpoints
        .route("/api/v1/activity-log", get(get_activity_log))

        .with_state(state)
}

// ============================================================================
// Health & Status Endpoints
// ============================================================================

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "version": "0.2.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// ============================================================================
// Host Endpoints
// ============================================================================

async fn list_hosts(
    State(_state): State<AppState>,
    Query(_params): Query<ListParams>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(HostListResponse {
        hosts: vec![],
        total: 0,
    }))
}

async fn get_host(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(ApiResponse::<Host>::error("Host not found".to_string()))
        .into_response()
}

// ============================================================================
// Service Endpoints
// ============================================================================

async fn list_services(
    State(_state): State<AppState>,
    Query(_params): Query<ListParams>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(ServiceListResponse {
        services: vec![],
        total: 0,
    }))
}

async fn get_service(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(ApiResponse::<Service>::error("Service not found".to_string()))
        .into_response()
}

// ============================================================================
// Session Endpoints
// ============================================================================

async fn list_sessions(
    State(_state): State<AppState>,
    Query(_params): Query<ListParams>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(SessionListResponse {
        sessions: vec![],
        total: 0,
    }))
}

async fn get_session(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(ApiResponse::<Session>::error("Session not found".to_string()))
        .into_response()
}

// ============================================================================
// Credential Endpoints
// ============================================================================

async fn list_credentials(
    State(_state): State<AppState>,
    Query(_params): Query<ListParams>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(CredentialListResponse {
        credentials: vec![],
        total: 0,
    }))
}

async fn get_credential(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(ApiResponse::<Credential>::error("Credential not found".to_string()))
        .into_response()
}

// ============================================================================
// Team Management Endpoints
// ============================================================================

async fn create_team(
    State(_state): State<AppState>,
    Json(req): Json<CreateTeamRequest>,
) -> impl IntoResponse {
    info!("Creating team: {}", req.name);
    let team_id = Uuid::new_v4();

    Json(ApiResponse::ok(json!({
        "id": team_id,
        "name": req.name,
        "description": req.description,
    })))
}

async fn get_team(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(ApiResponse::<Team>::error("Team not found".to_string()))
        .into_response()
}

async fn list_team_members(
    State(_state): State<AppState>,
    Path(_team_id): Path<Uuid>,
    Query(_params): Query<ListParams>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(json!({
        "members": [],
        "total": 0
    })))
}

async fn add_team_member(
    State(_state): State<AppState>,
    Path(_team_id): Path<Uuid>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(ApiResponse::ok(json!({}))))
}

async fn remove_team_member(
    State(_state): State<AppState>,
    Path((_team_id, _user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

// ============================================================================
// Workspace Management Endpoints
// ============================================================================

async fn create_workspace(
    State(_state): State<AppState>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> impl IntoResponse {
    info!("Creating workspace: {}", req.name);
    let workspace_id = Uuid::new_v4();

    (StatusCode::CREATED, Json(ApiResponse::ok(json!({
        "id": workspace_id,
        "name": req.name,
        "description": req.description,
        "visibility": req.visibility,
    }))))
}

async fn get_workspace(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(ApiResponse::<Workspace>::error("Workspace not found".to_string()))
        .into_response()
}

async fn list_workspace_permissions(
    State(_state): State<AppState>,
    Path(_workspace_id): Path<Uuid>,
    Query(_params): Query<ListParams>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(json!({
        "permissions": [],
        "total": 0
    })))
}

async fn update_workspace_permission(
    State(_state): State<AppState>,
    Path(_workspace_id): Path<Uuid>,
    Json(_req): Json<UpdatePermissionRequest>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(json!({})))
}

// ============================================================================
// C2 Framework Management Endpoints
// ============================================================================

async fn list_c2_servers(
    State(_state): State<AppState>,
    Query(_params): Query<ListParams>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(json!({
        "servers": [],
        "total": 0
    })))
}

async fn register_c2_server(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(ApiResponse::ok(json!({
        "id": Uuid::new_v4(),
    }))))
}

async fn connect_c2_server(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(json!({
        "connected": true,
    })))
}

// ============================================================================
// Activity Log Endpoints
// ============================================================================

async fn get_activity_log(
    State(_state): State<AppState>,
    Query(_params): Query<ListParams>,
) -> impl IntoResponse {
    Json(ApiResponse::ok(json!({
        "events": [],
        "total": 0
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_ok() {
        let resp: ApiResponse<String> = ApiResponse::ok("test".to_string());
        assert!(resp.success);
        assert_eq!(resp.data, Some("test".to_string()));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let resp = ApiResponse::<()>::error("error".to_string());
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert_eq!(resp.error, Some("error".to_string()));
    }
}
