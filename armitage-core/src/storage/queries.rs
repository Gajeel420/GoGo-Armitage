//! PostgreSQL query implementations

use crate::models::*;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Query implementations for hosts
pub struct HostQueries;

impl HostQueries {
    /// Inserts or updates a host record
    pub async fn upsert(pool: &PgPool, workspace_id: Uuid, host: &Host) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO hosts (
                id, workspace_id, address, ipv4, ipv6, mac_address, hostname,
                os_name, os_flavor, os_sp, os_lang, arch, purpose, info,
                comments, last_seen, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            ON CONFLICT(id) DO UPDATE SET updated_at = NOW()
            "#,
        )
        .bind(&host.id)
        .bind(&workspace_id)
        .bind(&host.address)
        .bind(&host.ipv4)
        .bind(&host.ipv6)
        .bind(&host.mac_address)
        .bind(&host.hostname)
        .bind(&host.os_name)
        .bind(&host.os_flavor)
        .bind(&host.os_sp)
        .bind(&host.os_lang)
        .bind(&host.arch)
        .bind(&host.purpose)
        .bind(&host.info)
        .bind(&host.comments)
        .bind(&host.last_seen)
        .bind(&host.created_at)
        .bind(&host.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Deletes a host record
    pub async fn delete(pool: &PgPool, host_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM hosts WHERE id = $1")
            .bind(&host_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// Query implementations for services
pub struct ServiceQueries;

impl ServiceQueries {
    /// Inserts or updates a service record
    pub async fn upsert(pool: &PgPool, service: &Service) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO services (
                id, host_id, port, proto, state, name, product, version,
                extrainfo, method, conf, info, comments, last_seen, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT(id) DO UPDATE SET updated_at = NOW()
            "#,
        )
        .bind(&service.id)
        .bind(&service.host_id)
        .bind(service.port as i32)
        .bind(service.proto.to_string())
        .bind(service.state.to_string())
        .bind(&service.name)
        .bind(&service.product)
        .bind(&service.version)
        .bind(&service.extrainfo)
        .bind(&service.method)
        .bind(&service.conf)
        .bind(&service.info)
        .bind(&service.comments)
        .bind(&service.last_seen)
        .bind(&service.created_at)
        .bind(&service.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Deletes a service record
    pub async fn delete(pool: &PgPool, service_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM services WHERE id = $1")
            .bind(&service_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// Query implementations for activity logs
pub struct ActivityLogQueries;

impl ActivityLogQueries {
    /// Inserts an activity log record
    pub async fn insert(pool: &PgPool, log: &ActivityLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO activity_logs (
                id, workspace_id, user_id, activity_type, resource_type,
                resource_id, action, details, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&log.id)
        .bind(&log.workspace_id)
        .bind(&log.user_id)
        .bind(log.activity_type.to_string())
        .bind(&log.resource_type)
        .bind(&log.resource_id)
        .bind(&log.action)
        .bind(&log.details)
        .bind(&log.timestamp)
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// Query implementations for workspaces
pub struct WorkspaceQueries;

impl WorkspaceQueries {
    /// Creates a new workspace
    pub async fn create(pool: &PgPool, workspace: &Workspace) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO workspaces (
                id, team_id, name, description, visibility, owner_id, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&workspace.id)
        .bind(&workspace.team_id)
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(workspace.visibility.to_string())
        .bind(&workspace.owner_id)
        .bind(&workspace.created_at)
        .bind(&workspace.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// Query implementations for C2 servers
pub struct C2ServerQueries;

impl C2ServerQueries {
    /// Registers a C2 server
    pub async fn register(pool: &PgPool, server: &C2Server) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO c2_servers (
                id, workspace_id, name, framework, host, port, username, password,
                connected, last_connected, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&server.id)
        .bind(&server.workspace_id)
        .bind(&server.name)
        .bind(server.framework.to_string())
        .bind(&server.host)
        .bind(server.port as i32)
        .bind(&server.username)
        .bind(&server.password)
        .bind(server.connected)
        .bind(&server.last_connected)
        .bind(&server.created_at)
        .bind(&server.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }
}
