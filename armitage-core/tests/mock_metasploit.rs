//! Mock Metasploit RPC server for integration testing
//!
//! Simulates Metasploit RPC responses for testing ingestors without
//! requiring a real Metasploit instance.

#[cfg(test)]
mod mock_metasploit {
    use chrono::Utc;
    use uuid::Uuid;
    use armitage_core::{Host, Service, Session, Credential, ServiceProto, ServiceState, SessionType, OriginType, PrivateType};

    /// Mock Metasploit RPC database
    pub struct MockMetasploitDb {
        hosts: Vec<Host>,
        services: Vec<Service>,
        sessions: Vec<Session>,
        credentials: Vec<Credential>,
    }

    impl MockMetasploitDb {
        pub fn new() -> Self {
            Self {
                hosts: Vec::new(),
                services: Vec::new(),
                sessions: Vec::new(),
                credentials: Vec::new(),
            }
        }

        pub fn add_host(&mut self, host: Host) {
            self.hosts.push(host);
        }

        pub fn add_service(&mut self, service: Service) {
            self.services.push(service);
        }

        pub fn add_session(&mut self, session: Session) {
            self.sessions.push(session);
        }

        pub fn add_credential(&mut self, cred: Credential) {
            self.credentials.push(cred);
        }

        pub fn get_hosts(&self) -> Vec<Host> {
            self.hosts.clone()
        }

        pub fn get_services(&self, host_id: Uuid) -> Vec<Service> {
            self.services
                .iter()
                .filter(|s| s.host_id == host_id)
                .cloned()
                .collect()
        }

        pub fn get_sessions(&self) -> Vec<Session> {
            self.sessions.clone()
        }

        pub fn get_credentials(&self) -> Vec<Credential> {
            self.credentials.clone()
        }
    }

    /// Populate with realistic sample data
    pub fn populate_sample_data(db: &mut MockMetasploitDb) {
        // Add hosts
        let host1 = Host {
            id: Uuid::new_v4(),
            address: "192.168.1.100".to_string(),
            ipv4: "192.168.1.100".to_string(),
            ipv6: None,
            mac_address: Some("00:0A:95:9D:68:16".to_string()),
            hostname: Some("webserver.internal".to_string()),
            os_name: Some("Linux".to_string()),
            os_flavor: Some("Ubuntu".to_string()),
            os_sp: Some("20.04".to_string()),
            os_lang: None,
            arch: Some("x86_64".to_string()),
            purpose: Some("Web Server".to_string()),
            info: None,
            comments: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let host2 = Host {
            id: Uuid::new_v4(),
            address: "192.168.1.50".to_string(),
            ipv4: "192.168.1.50".to_string(),
            ipv6: None,
            mac_address: Some("08:00:27:00:00:01".to_string()),
            hostname: Some("database.internal".to_string()),
            os_name: Some("Windows".to_string()),
            os_flavor: Some("Server 2019".to_string()),
            os_sp: None,
            os_lang: None,
            arch: Some("x86_64".to_string()),
            purpose: Some("Database Server".to_string()),
            info: None,
            comments: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        db.add_host(host1.clone());
        db.add_host(host2.clone());

        // Add services
        let service1 = Service {
            id: Uuid::new_v4(),
            host_id: host1.id,
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
        };

        let service2 = Service {
            id: Uuid::new_v4(),
            host_id: host1.id,
            port: 22,
            proto: ServiceProto::Tcp,
            state: ServiceState::Open,
            name: Some("ssh".to_string()),
            product: Some("OpenSSH".to_string()),
            version: Some("7.4".to_string()),
            extrainfo: None,
            method: Some("table".to_string()),
            conf: Some(10),
            info: None,
            comments: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let service3 = Service {
            id: Uuid::new_v4(),
            host_id: host2.id,
            port: 1433,
            proto: ServiceProto::Tcp,
            state: ServiceState::Open,
            name: Some("mssql".to_string()),
            product: Some("Microsoft SQL Server".to_string()),
            version: Some("2019".to_string()),
            extrainfo: None,
            method: Some("table".to_string()),
            conf: Some(10),
            info: None,
            comments: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        db.add_service(service1);
        db.add_service(service2);
        db.add_service(service3);

        // Add sessions
        let session1 = Session {
            id: Uuid::new_v4(),
            sid: 1,
            host_id: host1.id,
            session_type: SessionType::Meterpreter,
            platform: "linux/x86_64".to_string(),
            user: Some("www-data".to_string()),
            via_exploit: Some("exploit/linux/http/apache_struts_rce".to_string()),
            via_payload: Some("payload/linux/x86_64/meterpreter/reverse_tcp".to_string()),
            tunnel_peer: None,
            tunnel_local: None,
            target_host: Some("192.168.1.100".to_string()),
            target_port: Some(443),
            description: None,
            info: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            closed_at: None,
        };

        db.add_session(session1);

        // Add credentials
        let cred1 = Credential {
            id: Uuid::new_v4(),
            host_id: host1.id,
            service_id: None,
            origin_type: OriginType::Import,
            private_type: PrivateType::Password,
            private_data: "P@ssw0rd123".to_string(),
            public: Some("admin".to_string()),
            realm: Some("webserver.internal".to_string()),
            username: Some("admin".to_string()),
            password: Some("P@ssw0rd123".to_string()),
            ntlm_hash: None,
            lm_hash: None,
            ssh_key: None,
            jtr_format: None,
            source_id: None,
            source_type: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
        };

        let cred2 = Credential {
            id: Uuid::new_v4(),
            host_id: host2.id,
            service_id: None,
            origin_type: OriginType::Exploit,
            private_type: PrivateType::Password,
            private_data: "SQLAdmin123".to_string(),
            public: Some("sa".to_string()),
            realm: Some("database.internal".to_string()),
            username: Some("sa".to_string()),
            password: Some("SQLAdmin123".to_string()),
            ntlm_hash: None,
            lm_hash: None,
            ssh_key: None,
            jtr_format: None,
            source_id: None,
            source_type: None,
            last_seen: Utc::now(),
            created_at: Utc::now(),
        };

        db.add_credential(cred1);
        db.add_credential(cred2);
    }

    #[test]
    fn test_mock_metasploit_db() {
        let mut db = MockMetasploitDb::new();
        populate_sample_data(&mut db);

        assert_eq!(db.get_hosts().len(), 2);
        assert_eq!(db.get_sessions().len(), 1);
        assert_eq!(db.get_credentials().len(), 2);

        let host1 = &db.get_hosts()[0];
        let services = db.get_services(host1.id);
        assert_eq!(services.len(), 2);
    }

    #[test]
    fn test_mock_db_data_consistency() {
        let mut db = MockMetasploitDb::new();
        populate_sample_data(&mut db);

        let hosts = db.get_hosts();
        let services = db.get_services(hosts[0].id);

        // Verify relationships
        for service in services {
            assert_eq!(service.host_id, hosts[0].id);
        }

        // Verify credentials
        let creds = db.get_credentials();
        assert!(creds.iter().all(|c| !c.username.as_ref().unwrap().is_empty()));
    }
}
