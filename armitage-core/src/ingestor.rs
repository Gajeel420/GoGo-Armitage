//! Data ingestion layer for receiving data from multiple C2 sources
//!
//! Provides a unified ingestion pipeline that supports multiple C2 frameworks
//! (Metasploit, Sliver) simultaneously, converting their native data formats
//! into internal models and publishing events for downstream processing.

use crate::events::{Event, EventBroker};
use crate::models::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub mod msf_ingestor;
pub mod sliver_ingestor;

pub use msf_ingestor::{MetasploitIngestor, MsfIngestorConfig};
pub use sliver_ingestor::{SliverIngestor, SliverIngestorConfig};

/// Main data ingestor that dispatches events from any source
pub struct DataIngestor {
    event_broker: EventBroker,
}

impl DataIngestor {
    /// Creates a new data ingestor
    pub fn new(event_broker: EventBroker) -> Self {
        Self { event_broker }
    }

    /// Returns a reference to the event broker
    pub fn event_broker(&self) -> &EventBroker {
        &self.event_broker
    }

    /// Ingests a host discovery event
    pub async fn ingest_host(&self, host: Host) -> Result<()> {
        self.event_broker
            .publish(Event::HostAdded(Box::new(host)))
            .await?;
        Ok(())
    }

    /// Ingests a service discovery event
    pub async fn ingest_service(&self, service: Service) -> Result<()> {
        self.event_broker
            .publish(Event::ServiceAdded(Box::new(service)))
            .await?;
        Ok(())
    }

    /// Ingests a session event
    pub async fn ingest_session(&self, session: Session) -> Result<()> {
        self.event_broker
            .publish(Event::SessionOpened(Box::new(session)))
            .await?;
        Ok(())
    }

    /// Ingests a credential discovery
    pub async fn ingest_credential(&self, credential: Credential) -> Result<()> {
        self.event_broker
            .publish(Event::CredentialFound(Box::new(credential)))
            .await?;
        Ok(())
    }
}

/// Multi-C2 ingestor orchestrator
///
/// Manages multiple C2 framework ingestors and coordinates their lifecycle.
pub struct IngestorOrchestrator {
    event_broker: EventBroker,
    msf_ingestors: Vec<Arc<MetasploitIngestor>>,
    sliver_ingestors: Vec<Arc<SliverIngestor>>,
    running: Arc<Mutex<bool>>,
}

impl IngestorOrchestrator {
    /// Creates a new orchestrator
    pub fn new(event_broker: EventBroker) -> Self {
        Self {
            event_broker,
            msf_ingestors: Vec::new(),
            sliver_ingestors: Vec::new(),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Registers a Metasploit ingestor
    pub fn add_metasploit(&mut self, config: MsfIngestorConfig) -> Arc<MetasploitIngestor> {
        let ingestor = Arc::new(MetasploitIngestor::new(config, self.event_broker.clone()));
        self.msf_ingestors.push(ingestor.clone());
        ingestor
    }

    /// Registers a Sliver ingestor
    pub fn add_sliver(&mut self, config: SliverIngestorConfig) -> Arc<SliverIngestor> {
        let ingestor = Arc::new(SliverIngestor::new(config, self.event_broker.clone()));
        self.sliver_ingestors.push(ingestor.clone());
        ingestor
    }

    /// Starts all registered ingestors concurrently
    pub async fn start_all(&self) -> Result<()> {
        *self.running.lock().await = true;
        info!(
            "Starting {} Metasploit + {} Sliver ingestors",
            self.msf_ingestors.len(),
            self.sliver_ingestors.len()
        );

        let mut handles = Vec::new();

        // Start Metasploit ingestors
        for ingestor in &self.msf_ingestors {
            let ingestor = ingestor.clone();
            handles.push(tokio::spawn(async move {
                if let Err(e) = ingestor.connect().await {
                    warn!("Failed to connect Metasploit ingestor: {}", e);
                    return;
                }
                if let Err(e) = ingestor.start().await {
                    warn!("Metasploit ingestor error: {}", e);
                }
            }));
        }

        // Start Sliver ingestors
        for ingestor in &self.sliver_ingestors {
            let ingestor = ingestor.clone();
            handles.push(tokio::spawn(async move {
                if let Err(e) = ingestor.connect().await {
                    warn!("Failed to connect Sliver ingestor: {}", e);
                    return;
                }
                if let Err(e) = ingestor.start().await {
                    warn!("Sliver ingestor error: {}", e);
                }
            }));
        }

        // Wait for all ingestors
        for handle in handles {
            handle.await.ok();
        }

        Ok(())
    }

    /// Stops all registered ingestors
    pub async fn stop_all(&self) {
        info!("Stopping all ingestors");
        *self.running.lock().await = false;

        for ingestor in &self.msf_ingestors {
            ingestor.stop().await;
        }
        for ingestor in &self.sliver_ingestors {
            ingestor.stop().await;
        }
    }

    /// Returns total count of registered ingestors
    pub fn ingestor_count(&self) -> usize {
        self.msf_ingestors.len() + self.sliver_ingestors.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let broker = EventBroker::new(100);
        let orchestrator = IngestorOrchestrator::new(broker);
        assert_eq!(orchestrator.ingestor_count(), 0);
    }

    #[test]
    fn test_orchestrator_add_ingestors() {
        let broker = EventBroker::new(100);
        let mut orchestrator = IngestorOrchestrator::new(broker);

        orchestrator.add_metasploit(MsfIngestorConfig::default());
        orchestrator.add_sliver(SliverIngestorConfig::default());

        assert_eq!(orchestrator.ingestor_count(), 2);
    }

    #[tokio::test]
    async fn test_data_ingestor_publishes_events() {
        let broker = EventBroker::new(100);
        let mut rx = broker.subscribe();
        let ingestor = DataIngestor::new(broker);

        let host = Host {
            id: uuid::Uuid::new_v4(),
            address: "10.0.0.1".to_string(),
            ipv4: "10.0.0.1".to_string(),
            ipv6: None,
            mac_address: None,
            hostname: Some("target-1".to_string()),
            os_name: Some("Linux".to_string()),
            os_flavor: None,
            os_sp: None,
            os_lang: None,
            arch: Some("x86_64".to_string()),
            purpose: None,
            info: None,
            comments: None,
            last_seen: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        ingestor.ingest_host(host).await.unwrap();

        let event = rx.recv().await.unwrap();
        assert_eq!(event.event_type(), "host:added");
    }
}
