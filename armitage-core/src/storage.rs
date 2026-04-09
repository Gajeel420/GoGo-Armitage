//! Storage layer for persisting data to PostgreSQL
//!
//! Provides database access and query operations for hosts, services,
//! sessions, credentials, routes, and team collaboration data.

pub mod postgres;
pub mod queries;

pub use postgres::PostgresStorage;
