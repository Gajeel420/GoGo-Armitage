//! Integration tests for the complete data pipeline
//!
//! Tests the flow from C2 frameworks through ingestors, processors, and storage.

#[cfg(test)]
mod integration_tests {
    use armitage_core::{
        EventBroker, DataIngestor, DataAnalyzer, Host, Service, Session, Credential,
        OriginType, PrivateType, SessionType, ServiceProto, ServiceState,
        HostEnrichmentProcessor, CredentialDeduplicator,
    };
    use chrono::Utc;
    use uuid::Uuid;

    /// Test data fixtures
    mod fixtures {
        use super::*;

        pub fn sample_host() -> Host {
            Host {
                id: Uuid::new_v4(),
                address: "192.168.1.100".to_string(),
                ipv4: "192.168.1.100".to_string(),
                ipv6: None,
                mac_address: Some("00:0A:95:9D:68:16".to_string()),
                hostname: Some("target.local".to_string()),
                os_name: Some("Linux".to_string()),
                os_flavor: Some("Ubuntu".to_string()),
                os_sp: Some("20.04".to_string()),
                os_lang: None,
                arch: Some("x86_64".to_string()),
                purpose: Some("Web Server".to_string()),
                info: None,
                comments: Some("Test target".to_string()),
                last_seen: Utc::now(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }

        pub fn sample_service(host_id: Uuid) -> Service {
            Service {
                id: Uuid::new_v4(),
                host_id,
                port: 443,
                proto: ServiceProto::Tcp,
                state: ServiceState::Open,
                name: Some("https".to_string()),
                product: Some("nginx".to_string()),
                version: Some("1.18.0".to_string()),
                extrainfo: Some("Ubuntu".to_string()),
                method: Some("table".to_string()),
                conf: Some(10),
                info: None,
                comments: None,
                last_seen: Utc::now(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }

        pub fn sample_session(host_id: Uuid) -> Session {
            Session {
                id: Uuid::new_v4(),
                sid: 1,
                host_id,
                session_type: SessionType::Meterpreter,
                platform: "linux/x86_64".to_string(),
                user: Some("www-data".to_string()),
                via_exploit: Some("exploit/linux/http/apache_struts_rce".to_string()),
                via_payload: Some("payload/linux/x86_64/meterpreter/reverse_tcp".to_string()),
                tunnel_peer: None,
                tunnel_local: None,
                target_host: Some("192.168.1.100".to_string()),
                target_port: Some(443),
                description: Some("Test meterpreter session".to_string()),
                info: None,
                last_seen: Utc::now(),
                created_at: Utc::now(),
                closed_at: None,
            }
        }

        pub fn sample_credential(host_id: Uuid) -> Credential {
            Credential {
                id: Uuid::new_v4(),
                host_id,
                service_id: None,
                origin_type: OriginType::Import,
                private_type: PrivateType::Password,
                private_data: "SecurePassword123!".to_string(),
                public: Some("admin".to_string()),
                realm: Some("target.local".to_string()),
                username: Some("admin".to_string()),
                password: Some("SecurePassword123!".to_string()),
                ntlm_hash: None,
                lm_hash: None,
                ssh_key: None,
                jtr_format: None,
                source_id: None,
                source_type: None,
                last_seen: Utc::now(),
                created_at: Utc::now(),
            }
        }
    }

    #[tokio::test]
    async fn test_event_broker_integration() {
        let broker = EventBroker::new(100);
        let ingestor = DataIngestor::new(broker.clone());
        let mut subscriber = broker.subscribe();

        let host = fixtures::sample_host();
        let host_id = host.id;

        // Ingest a host
        ingestor.ingest_host(host).await.unwrap();

        // Verify event was published
        if let Ok(event) = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            subscriber.recv(),
        )
        .await
        {
            assert!(event.is_ok());
            let event = event.unwrap();
            assert_eq!(event.event_type(), "host:added");
        }
    }

    #[tokio::test]
    async fn test_ingestor_with_multiple_data_types() {
        let broker = EventBroker::new(100);
        let ingestor = DataIngestor::new(broker.clone());
        let mut subscriber = broker.subscribe();

        let host = fixtures::sample_host();
        let host_id = host.id;
        let service = fixtures::sample_service(host_id);
        let session = fixtures::sample_session(host_id);
        let credential = fixtures::sample_credential(host_id);

        // Ingest all data types
        ingestor.ingest_host(host).await.unwrap();
        ingestor.ingest_service(service).await.unwrap();
        ingestor.ingest_session(session).await.unwrap();
        ingestor.ingest_credential(credential).await.unwrap();

        // Verify events were published
        let mut event_count = 0;
        for _ in 0..4 {
            if let Ok(event) = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                subscriber.recv(),
            )
            .await
            {
                if event.is_ok() {
                    event_count += 1;
                }
            }
        }

        assert_eq!(event_count, 4, "Expected 4 events to be published");
    }

    #[tokio::test]
    async fn test_stream_processor_pipeline() {
        let broker = EventBroker::new(100);
        let ingestor = DataIngestor::new(broker.clone());

        // Create processors
        let enricher = HostEnrichmentProcessor::new(broker.clone());
        let deduplicator = CredentialDeduplicator::new();

        // Ingest data
        let host = fixtures::sample_host();
        let credential = fixtures::sample_credential(host.id);

        // Process through enricher
        ingestor.ingest_host(host.clone()).await.unwrap();
        enricher
            .process(&armitage_core::Event::HostAdded(Box::new(host)))
            .await
            .unwrap();

        // Process through deduplicator
        deduplicator
            .process(&armitage_core::Event::CredentialFound(Box::new(credential)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_complete_data_flow() {
        // Setup
        let broker = EventBroker::new(100);
        let ingestor = DataIngestor::new(broker.clone());
        let analyzer = DataAnalyzer::new(broker.clone());

        // Create sample data
        let host = fixtures::sample_host();
        let host_id = host.id;
        let service = fixtures::sample_service(host_id);
        let session = fixtures::sample_session(host_id);

        // Simulate complete flow
        ingestor.ingest_host(host.clone()).await.unwrap();
        ingestor.ingest_service(service.clone()).await.unwrap();
        ingestor.ingest_session(session.clone()).await.unwrap();

        // Verify analyzer can process session
        let analysis = analyzer.analyze_session(&session).await.unwrap();
        assert_eq!(analysis.risk_score, 0.8); // Meterpreter = HIGH
        assert!(analysis.pivoting_opportunity);
    }

    #[tokio::test]
    async fn test_subscriber_count_tracking() {
        let broker = EventBroker::new(100);
        assert_eq!(broker.subscriber_count(), 0);

        let _sub1 = broker.subscribe();
        assert_eq!(broker.subscriber_count(), 1);

        let _sub2 = broker.subscribe();
        assert_eq!(broker.subscriber_count(), 2);
    }
}
