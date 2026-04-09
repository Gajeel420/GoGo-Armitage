//! Real-time data analysis and streaming processor pipeline
//!
//! Subscribes to the event broker and processes incoming data through
//! enrichment, correlation, deduplication, and storage pipelines.
//! Supports multi-workspace data isolation and team broadcasting.

use crate::events::{Event, EventBroker};
use crate::models::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Main data analyzer and stream processor
pub struct DataAnalyzer {
    event_broker: EventBroker,
    processors: Vec<Box<dyn StreamProcessor>>,
}

impl DataAnalyzer {
    /// Creates a new data analyzer
    pub fn new(event_broker: EventBroker) -> Self {
        Self {
            event_broker,
            processors: Vec::new(),
        }
    }

    /// Registers a stream processor
    pub fn add_processor(&mut self, processor: Box<dyn StreamProcessor>) {
        self.processors.push(processor);
    }

    /// Starts processing events from the broker
    pub async fn start(&self) -> Result<()> {
        let mut rx = self.event_broker.subscribe();
        info!("Data analyzer started with {} processors", self.processors.len());

        loop {
            match rx.recv().await {
                Ok(event) => {
                    for processor in &self.processors {
                        if let Err(e) = processor.process(&event).await {
                            warn!("Processor error: {}", e);
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Analyzer lagged by {} events", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    info!("Event channel closed, analyzer stopping");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Enriches a host with additional information
    pub async fn enrich_host(&self, host: Host) -> Result<Host> {
        // Enrichment: reverse DNS, MAC vendor lookup, OS fingerprint
        Ok(host)
    }

    /// Correlates a service with known vulnerabilities
    pub async fn correlate_service(&self, _service: &Service) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Analyzes session activity for risk scoring
    pub async fn analyze_session(&self, session: &Session) -> Result<SessionAnalysis> {
        let risk_score = match session.session_type {
            SessionType::Meterpreter => 0.8,
            SessionType::Shell => 0.5,
            SessionType::VNC | SessionType::RDP => 0.6,
            SessionType::Other => 0.3,
        };

        let pivoting_opportunity = session.via_exploit.is_some();

        Ok(SessionAnalysis {
            session_id: session.id,
            risk_score,
            pivoting_opportunity,
        })
    }
}

/// Analysis result for a session
#[derive(Debug, Clone)]
pub struct SessionAnalysis {
    pub session_id: Uuid,
    pub risk_score: f64,
    pub pivoting_opportunity: bool,
}

/// Trait for stream processors in the analysis pipeline
#[async_trait]
pub trait StreamProcessor: Send + Sync {
    /// Process a single event
    async fn process(&self, event: &Event) -> Result<()>;

    /// Returns the processor name
    fn name(&self) -> &str;
}

/// Host enrichment processor
///
/// Enriches host data with reverse DNS, MAC vendor lookup, and OS detection.
pub struct HostEnrichmentProcessor {
    event_broker: EventBroker,
}

impl HostEnrichmentProcessor {
    pub fn new(event_broker: EventBroker) -> Self {
        Self { event_broker }
    }
}

#[async_trait]
impl StreamProcessor for HostEnrichmentProcessor {
    async fn process(&self, event: &Event) -> Result<()> {
        if let Event::HostAdded(host) = event {
            debug!("Enriching host: {}", host.ipv4);
            // In production: perform reverse DNS, MAC vendor lookup, etc.
            // For now, publish the enriched host as an update
            self.event_broker
                .publish(Event::HostUpdated(host.clone()))
                .await
                .ok();
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "host_enrichment"
    }
}

/// Service correlation processor
///
/// Correlates services with known vulnerability databases.
pub struct ServiceCorrelationProcessor {
    event_broker: EventBroker,
}

impl ServiceCorrelationProcessor {
    pub fn new(event_broker: EventBroker) -> Self {
        Self { event_broker }
    }
}

#[async_trait]
impl StreamProcessor for ServiceCorrelationProcessor {
    async fn process(&self, event: &Event) -> Result<()> {
        if let Event::ServiceAdded(service) = event {
            debug!(
                "Correlating service: {}:{} ({})",
                service.host_id,
                service.port,
                service.name.as_deref().unwrap_or("unknown")
            );
            // In production: match against CVE database, known exploits, etc.
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "service_correlation"
    }
}

/// Credential deduplication processor
///
/// Deduplicates credentials and tracks unique credential pairs.
pub struct CredentialDeduplicator {
    seen: Arc<Mutex<HashMap<String, Uuid>>>,
}

impl CredentialDeduplicator {
    pub fn new() -> Self {
        Self {
            seen: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn credential_key(cred: &Credential) -> String {
        format!(
            "{}:{}:{}",
            cred.username.as_deref().unwrap_or(""),
            cred.private_data,
            cred.host_id
        )
    }
}

#[async_trait]
impl StreamProcessor for CredentialDeduplicator {
    async fn process(&self, event: &Event) -> Result<()> {
        if let Event::CredentialFound(cred) = event {
            let key = Self::credential_key(cred);
            let mut seen = self.seen.lock().await;

            if seen.contains_key(&key) {
                debug!("Duplicate credential skipped: {}", key);
            } else {
                seen.insert(key, cred.id);
                info!(
                    "Unique credential tracked: user={}",
                    cred.username.as_deref().unwrap_or("unknown")
                );
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "credential_deduplicator"
    }
}

/// Session risk analyzer processor
///
/// Scores sessions by risk level and identifies pivoting opportunities.
pub struct SessionRiskAnalyzer {
    event_broker: EventBroker,
}

impl SessionRiskAnalyzer {
    pub fn new(event_broker: EventBroker) -> Self {
        Self { event_broker }
    }
}

#[async_trait]
impl StreamProcessor for SessionRiskAnalyzer {
    async fn process(&self, event: &Event) -> Result<()> {
        if let Event::SessionOpened(session) = event {
            let risk = match session.session_type {
                SessionType::Meterpreter => "HIGH",
                SessionType::Shell => "MEDIUM",
                SessionType::VNC | SessionType::RDP => "MEDIUM",
                SessionType::Other => "LOW",
            };

            info!(
                "Session risk: sid={} type={} risk={} platform={}",
                session.sid, session.session_type, risk, session.platform
            );
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "session_risk_analyzer"
    }
}

/// Storage persistence processor
///
/// Persists events to the database storage layer.
pub struct StoragePersistenceProcessor {
    // In production, this would hold a reference to PostgresStorage
    event_count: Arc<Mutex<u64>>,
}

impl StoragePersistenceProcessor {
    pub fn new() -> Self {
        Self {
            event_count: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn events_processed(&self) -> u64 {
        *self.event_count.lock().await
    }
}

#[async_trait]
impl StreamProcessor for StoragePersistenceProcessor {
    async fn process(&self, event: &Event) -> Result<()> {
        let mut count = self.event_count.lock().await;
        *count += 1;

        match event {
            Event::HostAdded(host) => {
                debug!("Persisting host: {} ({})", host.ipv4, host.id);
                // storage.upsert_host(host).await?;
            }
            Event::ServiceAdded(service) => {
                debug!("Persisting service: {}:{}", service.host_id, service.port);
                // storage.upsert_service(service).await?;
            }
            Event::SessionOpened(session) => {
                debug!("Persisting session: sid={}", session.sid);
                // storage.add_session(session).await?;
            }
            Event::SessionClosed(id) => {
                debug!("Closing session: {}", id);
                // storage.close_session(id).await?;
            }
            Event::CredentialFound(cred) => {
                debug!("Persisting credential: {}", cred.id);
                // storage.add_credential(cred).await?;
            }
            _ => {}
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "storage_persistence"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_analysis() {
        let broker = EventBroker::new(100);
        let analyzer = DataAnalyzer::new(broker);

        let session = Session {
            id: Uuid::new_v4(),
            sid: 1,
            host_id: Uuid::new_v4(),
            session_type: SessionType::Meterpreter,
            platform: "windows/x64".to_string(),
            user: Some("SYSTEM".to_string()),
            via_exploit: Some("exploit/windows/smb/ms17_010".to_string()),
            via_payload: Some("windows/x64/meterpreter/reverse_tcp".to_string()),
            tunnel_peer: None,
            tunnel_local: None,
            target_host: Some("10.0.0.5".to_string()),
            target_port: Some(445),
            description: None,
            info: None,
            last_seen: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            closed_at: None,
        };

        let analysis = analyzer.analyze_session(&session).await.unwrap();
        assert_eq!(analysis.risk_score, 0.8);
        assert!(analysis.pivoting_opportunity);
    }

    #[tokio::test]
    async fn test_credential_deduplicator() {
        let dedup = CredentialDeduplicator::new();

        let cred = Credential {
            id: Uuid::new_v4(),
            host_id: Uuid::new_v4(),
            service_id: None,
            origin_type: OriginType::Import,
            private_type: PrivateType::Password,
            private_data: "hunter2".to_string(),
            public: None,
            realm: None,
            username: Some("admin".to_string()),
            password: Some("hunter2".to_string()),
            ntlm_hash: None,
            lm_hash: None,
            ssh_key: None,
            jtr_format: None,
            source_id: None,
            source_type: None,
            last_seen: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };

        let event = Event::CredentialFound(Box::new(cred.clone()));

        // First time - should be new
        dedup.process(&event).await.unwrap();
        assert_eq!(dedup.seen.lock().await.len(), 1);

        // Second time - should be deduplicated
        dedup.process(&event).await.unwrap();
        assert_eq!(dedup.seen.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn test_storage_persistence_counter() {
        let persistence = StoragePersistenceProcessor::new();

        let host = Host {
            id: Uuid::new_v4(),
            address: "10.0.0.1".to_string(),
            ipv4: "10.0.0.1".to_string(),
            ipv6: None,
            mac_address: None,
            hostname: None,
            os_name: None,
            os_flavor: None,
            os_sp: None,
            os_lang: None,
            arch: None,
            purpose: None,
            info: None,
            comments: None,
            last_seen: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        persistence
            .process(&Event::HostAdded(Box::new(host)))
            .await
            .unwrap();
        assert_eq!(persistence.events_processed().await, 1);
    }
}
