//! Armitage API - REST and gRPC endpoints for data access and team collaboration

pub mod rest;
pub mod grpc;

pub use rest::{create_rest_app, AppState, ApiResponse};
pub use grpc::{GrpcServer, WorkspaceEventService, TeamSyncService};

