//! Armitage API - REST and gRPC endpoints for data access

pub mod rest;
pub mod grpc;

pub use rest::create_rest_app;
pub use grpc::create_grpc_server;
