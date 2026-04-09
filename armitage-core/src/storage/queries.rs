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
            ON CONFLICT(id) DO UPDATE SET
                hostname = EXCLUDED.hostname,
                os_name = EXCLUDED.os_name,
                os_flavor = EXCLUDED.os_flavor,
                last_seen = EXCLUDED.last_seen,
                updated_at = NOW()
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

    /// Retrieves a host by ID
    pub async fn get_by_id(pool: &PgPool, host_id: Uuid) -> Result<Option<Host>> {
        let row = sqlx::query_as::<_, (
            Uuid, String, String, Option<String>, Option<String>, Option<String>, Option<String>,
            Option<String>, Option<String>, Option<String>, Option<String>, Option<String>,
            Option<String>, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>
        )>(
            r#"
            SELECT id, address, ipv4, ipv6, mac_address, hostname, os_name, os_flavor,
                   os_sp, os_lang, arch, purpose, info, comments, last_seen, created_at, updated_at
            FROM hosts WHERE id = $1
            "#,
        )
        .bind(&host_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| Host {
            id: r.0,
            address: r.1,
            ipv4: r.2,
            ipv6: r.3,
            mac_address: r.4,
            hostname: r.5,
            os_name: r.6,
            os_flavor: r.7,
            os_sp: r.8,
            os_lang: r.9,
            arch: r.10,
            purpose: r.11,
            info: r.12,
            comments: r.13,
            last_seen: r.14,
            created_at: r.15,
            updated_at: r.16,
        }))
    }

    /// Counts hosts in a workspace
    pub async fn count_in_workspace(pool: &PgPool, workspace_id: Uuid) -> Result<i64> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM hosts WHERE workspace_id = $1"
        )
        .bind(&workspace_id)
        .fetch_one(pool)
        .await?;

        Ok(row)
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
            ON CONFLICT(id) DO UPDATE SET
                state = EXCLUDED.state,
                version = EXCLUDED.version,
                last_seen = EXCLUDED.last_seen,
                updated_at = NOW()
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

    /// Counts services for a host
    pub async fn count_for_host(pool: &PgPool, host_id: Uuid) -> Result<i64> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM services WHERE host_id = $1"
        )
        .bind(&host_id)
        .fetch_one(pool)
        .await?;

        Ok(row)
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

/// Query implementations for sessions
pub struct SessionQueries;

impl SessionQueries {
    /// Inserts or updates a session record
    pub async fn upsert(pool: &PgPool, session: &Session) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, sid, host_id, session_type, platform, user_name,
                via_exploit, via_payload, tunnel_peer, tunnel_local,
                target_host, target_port, description, info,
                last_seen, created_at, closed_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT(id) DO UPDATE SET
                last_seen = EXCLUDED.last_seen,
                closed_at = EXCLUDED.closed_at
            "#,
        )
        .bind(&session.id)
        .bind(session.sid as i32)
        .bind(&session.host_id)
        .bind(session.session_type.to_string())
        .bind(&session.platform)
        .bind(&session.user)
        .bind(&session.via_exploit)
        .bind(&session.via_payload)
        .bind(&session.tunnel_peer)
        .bind(&session.tunnel_local)
        .bind(&session.target_host)
        .bind(session.target_port.map(|p| p as i32))
        .bind(&session.description)
        .bind(&session.info)
        .bind(&session.last_seen)
        .bind(&session.created_at)
        .bind(&session.closed_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Counts active sessions
    pub async fn count_active(pool: &PgPool) -> Result<i64> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sessions WHERE closed_at IS NULL"
        )
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    /// Deletes a session record
    pub async fn delete(pool: &PgPool, session_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(&session_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// Query implementations for credentials
pub struct CredentialQueries;

impl CredentialQueries {
    /// Inserts or updates a credential record
    pub async fn upsert(pool: &PgPool, credential: &Credential) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO credentials (
                id, host_id, service_id, origin_type, private_type,
                private_data, public, realm, username, password,
                ntlm_hash, lm_hash, ssh_key, jtr_format,
                source_id, source_type, last_seen, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            ON CONFLICT(id) DO UPDATE SET
                last_seen = EXCLUDED.last_seen
            "#,
        )
        .bind(&credential.id)
        .bind(&credential.host_id)
        .bind(&credential.service_id)
        .bind(credential.origin_type.to_string())
        .bind(credential.private_type.to_string())
        .bind(&credential.private_data)
        .bind(&credential.public)
        .bind(&credential.realm)
        .bind(&credential.username)
        .bind(&credential.password)
        .bind(&credential.ntlm_hash)
        .bind(&credential.lm_hash)
        .bind(&credential.ssh_key)
        .bind(&credential.jtr_format)
        .bind(&credential.source_id)
        .bind(&credential.source_type)
        .bind(&credential.last_seen)
        .bind(&credential.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Counts unique credentials for a host
    pub async fn count_for_host(pool: &PgPool, host_id: Uuid) -> Result<i64> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM credentials WHERE host_id = $1"
        )
        .bind(&host_id)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    /// Deletes a credential record
    pub async fn delete(pool: &PgPool, credential_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM credentials WHERE id = $1")
            .bind(&credential_id)
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

    /// Counts logs for a workspace
    pub async fn count_for_workspace(pool: &PgPool, workspace_id: Uuid) -> Result<i64> {
        let row = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM activity_logs WHERE workspace_id = $1"
        )
        .bind(&workspace_id)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_module_compiles() {
        // This test just verifies the module compiles
        // Actual database tests would require a running PostgreSQL instance
    }
}

