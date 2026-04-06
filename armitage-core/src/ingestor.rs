//! Data ingestion layer for receiving data from various sources
//!
//! Handles ingestion from Metasploit RPC, PostgreSQL, and live sessions.

use crate::events::EventBroker;
use crate::models::*;
use anyhow::Result;

/// Main data ingestor
pub struct DataIngestor {
    event_broker: EventBroker,
}

impl DataIngestor {
    /// Creates a new data ingestor
    pub fn new(event_broker: EventBroker) -> Self {
        Self { event_broker }
    }

    /// Ingests a host discovery event
    pub async fn ingest_host(&self, host: Host) -> Result<()> {
        self.event_broker.publish(crate::events::Event::HostAdded(Box::new(host))).await?;
        Ok(())
    }

    /// Ingests a service discovery event
    pub async fn ingest_service(&self, service: Service) -> Result<()> {
        self.event_broker.publish(crate::events::Event::ServiceAdded(Box::new(service))).await?;
        Ok(())
    }

    /// Ingests a session event
    pub async fn ingest_session(&self, session: Session) -> Result<()> {
        self.event_broker.publish(crate::events::Event::SessionOpened(Box::new(session))).await?;
        Ok(())
    }

    /// Ingests a credential discovery
    pub async fn ingest_credential(&self, credential: Credential) -> Result<()> {
        self.event_broker.publish(crate::events::Event::CredentialFound(Box::new(credential))).await?;
        Ok(())
    }
}

pub mod msf_ingestor;
