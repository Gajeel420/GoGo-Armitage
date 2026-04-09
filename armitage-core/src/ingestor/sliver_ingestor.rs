//! Sliver gRPC data ingestor
//!
//! Connects to a Sliver C2 server via gRPC and ingests implant/beacon
//! data, converting it to internal models and publishing events.

use crate::c2::{C2Client, SliverClient};
use crate::events::{Event, EventBroker};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for the Sliver ingestor
#[derive(Debug, Clone)]
pub struct SliverIngestorConfig {
    pub host: String,
    pub port: u16,
    pub insecure: bool,
    pub poll_interval: Duration,
    pub workspace_id: Uuid,
}

impl Default for SliverIngestorConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 31337,
            insecure: false,
            poll_interval: Duration::from_secs(5),
            workspace_id: Uuid::new_v4(),
        }
    }
}

/// State tracker for Sliver data changes
struct SliverState {
    known_sessions: HashMap<u32, Uuid>,
    known_hosts: HashMap<String, Uuid>,
}

impl SliverState {
    fn new() -> Self {
        Self {
            known_sessions: HashMap::new(),
            known_hosts: HashMap::new(),
        }
    }
}

/// Sliver C2 data ingestor
pub struct SliverIngestor {
    config: SliverIngestorConfig,
    client: SliverClient,
    event_broker: EventBroker,
    state: Arc<Mutex<SliverState>>,
    running: Arc<Mutex<bool>>,
}

impl SliverIngestor {
    /// Creates a new Sliver ingestor
    pub fn new(config: SliverIngestorConfig, event_broker: EventBroker) -> Self {
        let client = SliverClient::new(
            config.host.clone(),
            config.port,
            config.insecure,
        );

        Self {
            config,
            client,
            event_broker,
            state: Arc::new(Mutex::new(SliverState::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Connects to the Sliver gRPC server
    pub async fn connect(&self) -> Result<()> {
        info!(
            "Connecting to Sliver at {}:{}",
            self.config.host, self.config.port
        );
        self.client.connect().await?;
        info!("Connected to Sliver successfully");
        Ok(())
    }

    /// Starts the polling/streaming loop
    pub async fn start(&self) -> Result<()> {
        if !self.client.is_connected().await? {
            return Err(anyhow!("Not connected to Sliver"));
        }

        *self.running.lock().await = true;
        info!(
            "Starting Sliver ingestor with {}s poll interval",
            self.config.poll_interval.as_secs()
        );

        let mut interval = time::interval(self.config.poll_interval);

        while *self.running.lock().await {
            interval.tick().await;

            if let Err(e) = self.poll_cycle().await {
                error!("Sliver poll cycle error: {}", e);
                self.event_broker
                    .publish(Event::Error(format!("Sliver poll error: {}", e)))
                    .await
                    .ok();
            }
        }

        info!("Sliver ingestor stopped");
        Ok(())
    }

    /// Stops the polling loop
    pub async fn stop(&self) {
        info!("Stopping Sliver ingestor");
        *self.running.lock().await = false;
    }

    /// Performs a single poll cycle
    async fn poll_cycle(&self) -> Result<()> {
        debug!("Starting Sliver poll cycle");

        let (sessions_result, hosts_result) = tokio::join!(
            self.poll_sessions(),
            self.poll_hosts(),
        );

        if let Err(e) = sessions_result {
            warn!("Sliver session polling failed: {}", e);
        }
        if let Err(e) = hosts_result {
            warn!("Sliver host polling failed: {}", e);
        }

        debug!("Sliver poll cycle complete");
        Ok(())
    }

    /// Polls sessions (implants/beacons) from Sliver
    async fn poll_sessions(&self) -> Result<()> {
        let sessions = self.client.list_sessions().await?;
        let mut state = self.state.lock().await;

        let current_sids: Vec<u32> = sessions.iter().map(|s| s.sid).collect();

        // Detect closed sessions
        let closed: Vec<u32> = state
            .known_sessions
            .keys()
            .filter(|sid| !current_sids.contains(sid))
            .copied()
            .collect();

        for sid in closed {
            if let Some(uuid) = state.known_sessions.remove(&sid) {
                info!("Sliver session/implant closed: sid={}", sid);
                self.event_broker
                    .publish(Event::SessionClosed(uuid))
                    .await
                    .ok();
            }
        }

        // Detect new sessions
        for session in sessions {
            if !state.known_sessions.contains_key(&session.sid) {
                state.known_sessions.insert(session.sid, session.id);
                info!(
                    "New Sliver session: sid={} type={} on {}",
                    session.sid, session.session_type, session.platform
                );
                self.event_broker
                    .publish(Event::SessionOpened(Box::new(session)))
                    .await
                    .ok();
            }
        }

        Ok(())
    }

    /// Polls hosts from Sliver
    async fn poll_hosts(&self) -> Result<()> {
        let hosts = self.client.list_hosts().await?;
        let mut state = self.state.lock().await;

        for host in hosts {
            let key = host.ipv4.clone();
            if state.known_hosts.contains_key(&key) {
                self.event_broker
                    .publish(Event::HostUpdated(Box::new(host)))
                    .await
                    .ok();
            } else {
                state.known_hosts.insert(key, host.id);
                info!("New Sliver host discovered: {} ({})", host.ipv4, host.id);
                self.event_broker
                    .publish(Event::HostAdded(Box::new(host)))
                    .await
                    .ok();
            }
        }

        Ok(())
    }

    /// Returns whether the ingestor is running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    /// Returns count of tracked sessions
    pub async fn known_session_count(&self) -> usize {
        self.state.lock().await.known_sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliver_default_config() {
        let config = SliverIngestorConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 31337);
        assert!(!config.insecure);
    }

    #[tokio::test]
    async fn test_sliver_ingestor_creation() {
        let config = SliverIngestorConfig::default();
        let broker = EventBroker::new(100);
        let ingestor = SliverIngestor::new(config, broker);

        assert!(!ingestor.is_running().await);
        assert_eq!(ingestor.known_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_sliver_ingestor_start_without_connect() {
        let config = SliverIngestorConfig::default();
        let broker = EventBroker::new(100);
        let ingestor = SliverIngestor::new(config, broker);

        let result = ingestor.start().await;
        assert!(result.is_err());
    }
}
