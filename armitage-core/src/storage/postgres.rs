//! PostgreSQL storage implementation

use crate::models::*;
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;
use chrono::Utc;

/// PostgreSQL storage backend
pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    /// Creates a new PostgreSQL storage backend
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Initializes the database schema
    pub async fn init(&self) -> Result<()> {
        // Core data tables
        self.create_core_tables().await?;

        // Team collaboration tables
        self.create_team_tables().await?;

        // Create indexes for performance
        self.create_indexes().await?;

        Ok(())
    }

    async fn create_core_tables(&self) -> Result<()> {
        // Hosts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS hosts (
                id UUID PRIMARY KEY,
                workspace_id UUID,
                address VARCHAR(255) NOT NULL,
                ipv4 VARCHAR(15) NOT NULL,
                ipv6 VARCHAR(45),
                mac_address VARCHAR(17),
                hostname VARCHAR(255),
                os_name VARCHAR(255),
                os_flavor VARCHAR(255),
                os_sp VARCHAR(255),
                os_lang VARCHAR(255),
                arch VARCHAR(255),
                purpose TEXT,
                info TEXT,
                comments TEXT,
                last_seen TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE,
                updated_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Services table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS services (
                id UUID PRIMARY KEY,
                host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
                port INTEGER NOT NULL,
                proto VARCHAR(10) NOT NULL,
                state VARCHAR(20) NOT NULL,
                name VARCHAR(255),
                product VARCHAR(255),
                version VARCHAR(255),
                extrainfo TEXT,
                method VARCHAR(255),
                conf INTEGER,
                info TEXT,
                comments TEXT,
                last_seen TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE,
                updated_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id UUID PRIMARY KEY,
                sid INTEGER NOT NULL,
                host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
                session_type VARCHAR(20) NOT NULL,
                platform VARCHAR(255) NOT NULL,
                user_name VARCHAR(255),
                via_exploit VARCHAR(255),
                via_payload VARCHAR(255),
                tunnel_peer VARCHAR(255),
                tunnel_local VARCHAR(255),
                target_host VARCHAR(255),
                target_port INTEGER,
                description TEXT,
                info TEXT,
                last_seen TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE,
                closed_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Credentials table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS credentials (
                id UUID PRIMARY KEY,
                host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
                service_id UUID REFERENCES services(id) ON DELETE SET NULL,
                origin_type VARCHAR(20) NOT NULL,
                private_type VARCHAR(20) NOT NULL,
                private_data TEXT NOT NULL,
                public TEXT,
                realm VARCHAR(255),
                username VARCHAR(255),
                password TEXT,
                ntlm_hash VARCHAR(255),
                lm_hash VARCHAR(255),
                ssh_key TEXT,
                jtr_format VARCHAR(255),
                source_id UUID,
                source_type VARCHAR(255),
                last_seen TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Routes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS routes (
                id UUID PRIMARY KEY,
                session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                subnet VARCHAR(18) NOT NULL,
                netmask VARCHAR(18) NOT NULL,
                gateway VARCHAR(255) NOT NULL,
                metric INTEGER,
                created_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Loots table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loots (
                id UUID PRIMARY KEY,
                host_id UUID NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
                session_id UUID REFERENCES sessions(id) ON DELETE SET NULL,
                loot_type VARCHAR(255) NOT NULL,
                data BYTEA NOT NULL,
                filename VARCHAR(255),
                content_type VARCHAR(255),
                description TEXT,
                created_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_team_tables(&self) -> Result<()> {
        // Teams table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS teams (
                id UUID PRIMARY KEY,
                name VARCHAR(255) NOT NULL UNIQUE,
                description TEXT,
                owner_id UUID NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE,
                updated_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Workspaces table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspaces (
                id UUID PRIMARY KEY,
                team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                visibility VARCHAR(20) NOT NULL DEFAULT 'private',
                owner_id UUID NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE,
                updated_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Team members table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS team_members (
                id UUID PRIMARY KEY,
                team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                user_id UUID NOT NULL,
                role VARCHAR(20) NOT NULL DEFAULT 'viewer',
                joined_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Workspace permissions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspace_permissions (
                id UUID PRIMARY KEY,
                workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                user_id UUID NOT NULL,
                role VARCHAR(20) NOT NULL DEFAULT 'viewer',
                granted_at TIMESTAMP WITH TIME ZONE,
                granted_by UUID NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Shared sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS shared_sessions (
                id UUID PRIMARY KEY,
                session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                shared_by UUID NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Activity log table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS activity_logs (
                id UUID PRIMARY KEY,
                workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                user_id UUID NOT NULL,
                activity_type VARCHAR(50) NOT NULL,
                resource_type VARCHAR(100) NOT NULL,
                resource_id UUID,
                action TEXT NOT NULL,
                details TEXT,
                timestamp TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // C2 servers table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS c2_servers (
                id UUID PRIMARY KEY,
                workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                framework VARCHAR(50) NOT NULL,
                host VARCHAR(255) NOT NULL,
                port INTEGER NOT NULL,
                username VARCHAR(255),
                password VARCHAR(255),
                connected BOOLEAN DEFAULT FALSE,
                last_connected TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE,
                updated_at TIMESTAMP WITH TIME ZONE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_indexes(&self) -> Result<()> {
        // Host queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_hosts_workspace_id ON hosts(workspace_id);")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_hosts_ipv4 ON hosts(ipv4);")
            .execute(&self.pool)
            .await?;

        // Service queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_services_host_id ON services(host_id);")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_services_port ON services(port);")
            .execute(&self.pool)
            .await?;

        // Session queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_host_id ON sessions(host_id);")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_workspace_id ON sessions(host_id);")
            .execute(&self.pool)
            .await?;

        // Team queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_members_team_id ON team_members(team_id);")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_team_members_user_id ON team_members(user_id);")
            .execute(&self.pool)
            .await?;

        // Workspace queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workspaces_team_id ON workspaces(team_id);")
            .execute(&self.pool)
            .await?;

        // Permissions queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workspace_permissions_workspace_id ON workspace_permissions(workspace_id);")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workspace_permissions_user_id ON workspace_permissions(user_id);")
            .execute(&self.pool)
            .await?;

        // Activity log queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_activity_logs_workspace_id ON activity_logs(workspace_id);")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_activity_logs_user_id ON activity_logs(user_id);")
            .execute(&self.pool)
            .await?;

        // C2 servers queries
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_c2_servers_workspace_id ON c2_servers(workspace_id);")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Gets the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_postgres_connection() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_default();
        if !db_url.is_empty() {
            let storage = PostgresStorage::new(&db_url).await.unwrap();
            let _ = storage.init().await;
        }
    }
}
