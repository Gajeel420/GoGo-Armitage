//! Metasploit RPC data ingestor
//!
//! Polls the Metasploit RPC server at configurable intervals, converts
//! RPC responses into internal data models, and publishes events through
//! the event broker for downstream processing.

use crate::c2::{C2Client, MetasploitClient};
use crate::events::{Event, EventBroker};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for the Metasploit ingestor
#[derive(Debug, Clone)]
pub struct MsfIngestorConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub poll_interval: Duration,
    pub workspace_id: Uuid,
}

impl Default for MsfIngestorConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 55553,
            username: "msf".to_string(),
            password: String::new(),
            poll_interval: Duration::from_secs(5),
            workspace_id: Uuid::new_v4(),
        }
    }
}

/// State tracker for detecting changes between polls
struct IngestorState {
    known_hosts: HashMap<String, Uuid>,
    known_sessions: HashMap<u32, Uuid>,
    known_credentials: HashMap<String, Uuid>,
}

impl IngestorState {
    fn new() -> Self {
        Self {
            known_hosts: HashMap::new(),
            known_sessions: HashMap::new(),
            known_credentials: HashMap::new(),
        }
    }
}

/// Metasploit RPC data ingestor with polling loop
pub struct MetasploitIngestor {
    config: MsfIngestorConfig,
    client: MetasploitClient,
    event_broker: EventBroker,
    state: Arc<Mutex<IngestorState>>,
    running: Arc<Mutex<bool>>,
}

impl MetasploitIngestor {
    /// Creates a new Metasploit ingestor
    pub fn new(config: MsfIngestorConfig, event_broker: EventBroker) -> Self {
        let client = MetasploitClient::new(
            config.host.clone(),
            config.port,
            config.username.clone(),
            config.password.clone(),
        );

        Self {
            config,
            client,
            event_broker,
            state: Arc::new(Mutex::new(IngestorState::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Connects to the Metasploit RPC server
    pub async fn connect(&self) -> Result<()> {
        info!(
            "Connecting to Metasploit RPC at {}:{}",
            self.config.host, self.config.port
        );
        self.client.connect().await?;
        info!("Connected to Metasploit RPC successfully");
        Ok(())
    }

    /// Starts the polling loop
    pub async fn start(&self) -> Result<()> {
        if !self.client.is_connected().await? {
            return Err(anyhow!("Not connected to Metasploit RPC"));
        }

        *self.running.lock().await = true;
        info!(
            "Starting Metasploit ingestor with {}s poll interval",
            self.config.poll_interval.as_secs()
        );

        let mut interval = time::interval(self.config.poll_interval);

        while *self.running.lock().await {
            interval.tick().await;

            if let Err(e) = self.poll_cycle().await {
                error!("Poll cycle error: {}", e);
                self.event_broker
                    .publish(Event::Error(format!("MSF poll error: {}", e)))
                    .await
                    .ok();
            }
        }

        info!("Metasploit ingestor stopped");
        Ok(())
    }

    /// Stops the polling loop
    pub async fn stop(&self) {
        info!("Stopping Metasploit ingestor");
        *self.running.lock().await = false;
    }

    /// Performs a single poll cycle
    async fn poll_cycle(&self) -> Result<()> {
        debug!("Starting poll cycle");

        // Poll hosts, sessions, and credentials in parallel
        let (hosts_result, sessions_result, creds_result) = tokio::join!(
            self.poll_hosts(),
            self.poll_sessions(),
            self.poll_credentials(),
        );

        if let Err(e) = hosts_result {
            warn!("Host polling failed: {}", e);
        }
        if let Err(e) = sessions_result {
            warn!("Session polling failed: {}", e);
        }
        if let Err(e) = creds_result {
            warn!("Credential polling failed: {}", e);
        }

        debug!("Poll cycle complete");
        Ok(())
    }

    /// Polls hosts from Metasploit and publishes new/updated events
    async fn poll_hosts(&self) -> Result<()> {
        let hosts = self.client.list_hosts().await?;
        let mut state = self.state.lock().await;

        for host in hosts {
            let key = host.ipv4.clone();
            if state.known_hosts.contains_key(&key) {
                // Host already known - publish update
                self.event_broker
                    .publish(Event::HostUpdated(Box::new(host)))
                    .await
                    .ok();
            } else {
                // New host discovered
                state.known_hosts.insert(key, host.id);
                info!("New host discovered: {} ({})", host.ipv4, host.id);
                self.event_broker
                    .publish(Event::HostAdded(Box::new(host)))
                    .await
                    .ok();
            }
        }

        Ok(())
    }

    /// Polls sessions from Metasploit and publishes events
    async fn poll_sessions(&self) -> Result<()> {
        let sessions = self.client.list_sessions().await?;
        let mut state = self.state.lock().await;

        let current_sids: Vec<u32> = sessions.iter().map(|s| s.sid).collect();

        // Check for closed sessions
        let closed: Vec<u32> = state
            .known_sessions
            .keys()
            .filter(|sid| !current_sids.contains(sid))
            .copied()
            .collect();

        for sid in closed {
            if let Some(uuid) = state.known_sessions.remove(&sid) {
                info!("Session closed: sid={}", sid);
                self.event_broker
                    .publish(Event::SessionClosed(uuid))
                    .await
                    .ok();
            }
        }

        // Check for new sessions
        for session in sessions {
            if !state.known_sessions.contains_key(&session.sid) {
                state.known_sessions.insert(session.sid, session.id);
                info!(
                    "New session opened: sid={} type={} on {}",
                    session.sid,
                    session.session_type,
                    session.platform
                );
                self.event_broker
                    .publish(Event::SessionOpened(Box::new(session)))
                    .await
                    .ok();
            }
        }

        Ok(())
    }

    /// Polls credentials from Metasploit and publishes new ones
    async fn poll_credentials(&self) -> Result<()> {
        let credentials = self.client.list_credentials().await?;
        let mut state = self.state.lock().await;

        for cred in credentials {
            let key = format!(
                "{}:{}:{}",
                cred.username.as_deref().unwrap_or(""),
                cred.private_data,
                cred.host_id
            );

            if !state.known_credentials.contains_key(&key) {
                state.known_credentials.insert(key, cred.id);
                info!("New credential found: user={}", cred.username.as_deref().unwrap_or("unknown"));
                self.event_broker
                    .publish(Event::CredentialFound(Box::new(cred)))
                    .await
                    .ok();
            }
        }

        Ok(())
    }

    /// Returns the current number of known hosts
    pub async fn known_host_count(&self) -> usize {
        self.state.lock().await.known_hosts.len()
    }

    /// Returns the current number of known sessions
    pub async fn known_session_count(&self) -> usize {
        self.state.lock().await.known_sessions.len()
    }

    /// Returns whether the ingestor is running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MsfIngestorConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 55553);
        assert_eq!(config.poll_interval, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_ingestor_creation() {
        let config = MsfIngestorConfig::default();
        let broker = EventBroker::new(100);
        let ingestor = MetasploitIngestor::new(config, broker);

        assert!(!ingestor.is_running().await);
        assert_eq!(ingestor.known_host_count().await, 0);
        assert_eq!(ingestor.known_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_ingestor_start_without_connect() {
        let config = MsfIngestorConfig::default();
        let broker = EventBroker::new(100);
        let ingestor = MetasploitIngestor::new(config, broker);

        let result = ingestor.start().await;
        assert!(result.is_err());
    }
}
