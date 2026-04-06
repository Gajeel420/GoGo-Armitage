//! PostgreSQL storage implementation

use crate::models::*;
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

/// PostgreSQL storage backend
pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    /// Creates a new PostgreSQL storage backend
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Initializes the database schema
    pub async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS hosts (
                id UUID PRIMARY KEY,
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

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS services (
                id UUID PRIMARY KEY,
                host_id UUID NOT NULL REFERENCES hosts(id),
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

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id UUID PRIMARY KEY,
                sid INTEGER NOT NULL,
                host_id UUID NOT NULL REFERENCES hosts(id),
                session_type VARCHAR(20) NOT NULL,
                platform VARCHAR(255) NOT NULL,
                user VARCHAR(255),
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

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS credentials (
                id UUID PRIMARY KEY,
                host_id UUID NOT NULL REFERENCES hosts(id),
                service_id UUID REFERENCES services(id),
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

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS routes (
                id UUID PRIMARY KEY,
                session_id UUID NOT NULL REFERENCES sessions(id),
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

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS loots (
                id UUID PRIMARY KEY,
                host_id UUID NOT NULL REFERENCES hosts(id),
                session_id UUID REFERENCES sessions(id),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_postgres_connection() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_default();
        if !db_url.is_empty() {
            let _storage = PostgresStorage::new(&db_url).await.unwrap();
        }
    }
}
