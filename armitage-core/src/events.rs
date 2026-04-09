//! Event system for real-time data updates
//!
//! Provides a publish-subscribe system for broadcasting data changes
//! to multiple subscribers in real-time.

use crate::models::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Events that can be broadcast through the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Host events
    HostAdded(Box<Host>),
    HostUpdated(Box<Host>),
    HostDeleted(Uuid),

    // Service events
    ServiceAdded(Box<Service>),
    ServiceUpdated(Box<Service>),
    ServiceDeleted(Uuid),

    // Session events
    SessionOpened(Box<Session>),
    SessionUpdated(Box<Session>),
    SessionClosed(Uuid),

    // Credential events
    CredentialFound(Box<Credential>),
    CredentialUpdated(Box<Credential>),

    // Route events
    RouteAdded(Box<Route>),
    RouteRemoved(Uuid),

    // Loot events
    LootExtracted(Box<Loot>),

    // Team collaboration events
    TeamMemberJoined { team_id: Uuid, user_id: Uuid },
    TeamMemberLeft { team_id: Uuid, user_id: Uuid },
    WorkspaceCreated { workspace_id: Uuid, team_id: Uuid },
    WorkspaceUpdated { workspace_id: Uuid },
    SessionShared { session_id: Uuid, workspace_id: Uuid, shared_by: Uuid },

    // C2 framework events
    C2Connected { server_id: Uuid, framework: String },
    C2Disconnected { server_id: Uuid, framework: String },

    // System events
    ScanStarted { workspace_id: String },
    ScanCompleted { workspace_id: String },
    Error(String),
}

impl Event {
    /// Returns the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            Event::HostAdded(_) => "host:added",
            Event::HostUpdated(_) => "host:updated",
            Event::HostDeleted(_) => "host:deleted",
            Event::ServiceAdded(_) => "service:added",
            Event::ServiceUpdated(_) => "service:updated",
            Event::ServiceDeleted(_) => "service:deleted",
            Event::SessionOpened(_) => "session:opened",
            Event::SessionUpdated(_) => "session:updated",
            Event::SessionClosed(_) => "session:closed",
            Event::CredentialFound(_) => "credential:found",
            Event::CredentialUpdated(_) => "credential:updated",
            Event::RouteAdded(_) => "route:added",
            Event::RouteRemoved(_) => "route:removed",
            Event::LootExtracted(_) => "loot:extracted",
            Event::TeamMemberJoined { .. } => "team:member_joined",
            Event::TeamMemberLeft { .. } => "team:member_left",
            Event::WorkspaceCreated { .. } => "workspace:created",
            Event::WorkspaceUpdated { .. } => "workspace:updated",
            Event::SessionShared { .. } => "session:shared",
            Event::C2Connected { .. } => "c2:connected",
            Event::C2Disconnected { .. } => "c2:disconnected",
            Event::ScanStarted { .. } => "scan:started",
            Event::ScanCompleted { .. } => "scan:completed",
            Event::Error(_) => "error",
        }
    }
}

/// Publishes events to subscribers
#[derive(Clone)]
pub struct EventBroker {
    tx: broadcast::Sender<Arc<Event>>,
}

impl EventBroker {
    /// Creates a new event broker with the specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publishes an event to all subscribers
    pub async fn publish(&self, event: Event) -> Result<(), broadcast::error::SendError<Arc<Event>>> {
        self.tx.send(Arc::new(event)).map(|_| ())
    }

    /// Subscribes to all events
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<Event>> {
        self.tx.subscribe()
    }

    /// Returns the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_strings() {
        let event = Event::HostAdded(Box::new(Host {
            id: Uuid::new_v4(),
            address: "192.168.1.1".to_string(),
            ipv4: "192.168.1.1".to_string(),
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
        }));

        assert_eq!(event.event_type(), "host:added");
    }

    #[tokio::test]
    async fn test_event_broker() {
        let broker = EventBroker::new(100);
        let mut rx = broker.subscribe();

        let event = Event::HostAdded(Box::new(Host {
            id: Uuid::new_v4(),
            address: "192.168.1.1".to_string(),
            ipv4: "192.168.1.1".to_string(),
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
        }));

        broker.publish(event).await.unwrap();

        if let Ok(received) = rx.recv().await {
            assert!(matches!(*received, Event::HostAdded(_)));
        }
    }
}
