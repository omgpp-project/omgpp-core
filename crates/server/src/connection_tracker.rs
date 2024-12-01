use std::{collections::HashMap, net::IpAddr, time::Instant};

use bimap::BiHashMap;
use gns::{GnsConnection};
use omgpp_core::{ConnectionState, Endpoint};
use std::time::Duration;
use uuid::Uuid;


#[derive(Default, Debug)]
pub struct ConnectionTracker {
    connections: BiHashMap<Uuid, GnsConnection>,
    unverified_connections: HashMap<Uuid, Instant>,
    endpoints: BiHashMap<Uuid, Endpoint>,
    states: HashMap<Uuid,ConnectionState>,
    unverified_connection_expire_period: Duration
}

impl ConnectionTracker {
    pub fn new(unverified_connection_expire_period:Duration) -> ConnectionTracker{
        ConnectionTracker{
            unverified_connection_expire_period,
            ..Default::default()
        }
    }
    pub fn active_clients(&self) -> Vec<(Uuid, Endpoint)> {
        let endpoints = &self.endpoints;
        let active_endpoints = endpoints
            .into_iter()
            .filter(|item| !self.unverified_connections.contains_key(item.0))
            .map(|item| (item.0.clone(), item.1.clone()))
            .collect();
        active_endpoints
    }

    pub fn client_connection(&self, client: &Uuid) -> Option<GnsConnection> {
        self.connections
            .get_by_left(client)
            .map(|conn| conn.clone())
    }
    pub fn state(&self, client: &Uuid) -> ConnectionState {
        self.states
            .get(client)
            .cloned()
            .unwrap_or(ConnectionState::None)
    }
    pub fn client_endpoint(&self, client: &Uuid) -> Option<&Endpoint> {
        self.endpoints
            .get_by_left(client)
            .map(|conn| conn)
    }
    pub fn track_client_disconnected(&mut self, uuid: &Uuid) {
        if self.connections.contains_left(uuid) {
            self.connections.remove_by_left(uuid);
        }
        if self.endpoints.contains_left(uuid){
            self.endpoints.remove_by_left(uuid);
        }
        if self.unverified_connections.contains_key(uuid){
            self.unverified_connections.remove(uuid);
        }
        //TODO remove disconnected entries after some period; Prevent infinite collection growing
        self.states.insert(uuid.clone(), ConnectionState::Disconnected);
    }

    pub fn track_client_connected_unverified(&mut self, uuid: Uuid, endpoint:Endpoint,connection: GnsConnection) {
        if !self.connections.contains_left(&uuid){
            self.connections.insert(uuid,connection);
        }
        let now = Instant::now();
        println!("{:?} - {:?}",uuid,now);
        self.unverified_connections.insert(uuid, now);
        // TODO decide what todo when we have already associated endpoint
        let _old_endpoint = self.endpoints.insert(uuid, endpoint);   
        self.states.insert(uuid.clone(), ConnectionState::ConnectedUnverified);
    }
    pub fn track_client_connected(&mut self, uuid: Uuid, endpoint:Endpoint,connection: GnsConnection) {
        if self.unverified_connections.contains_key(&uuid){
            self.unverified_connections.remove(&uuid);
        }
        if !self.connections.contains_left(&uuid){
            self.connections.insert(uuid, connection);
        }
        // TODO decide what todo when we have already associated endpoint
        let _old_endpoint = self.endpoints.insert(uuid, endpoint);   
        self.states.insert(uuid.clone(), ConnectionState::Connected);
    }
    pub fn client_by_connection(&self, connection: &GnsConnection) -> Option<&Uuid> {
        self.connections.get_by_right(connection)
    }
    pub fn active_connections(&self) -> impl Iterator<Item = GnsConnection> + '_ {
        let connections = &self.connections;
        connections
            .into_iter()
            .filter(|item| !self.unverified_connections.contains_key(item.0))
            .map(|item| item.1.clone())
            .into_iter()
    }
    pub fn expired_unverified_connections(&self) ->impl Iterator<Item = GnsConnection> + '_ {
        let now = Instant::now();
        let expiring_period =self.unverified_connection_expire_period.clone();

        let unverified_connections = &self.unverified_connections;
        unverified_connections.into_iter()
            .filter(move |item| {
                let diff = now - *item.1;
                diff > expiring_period
            })
            .map(|item| self.connections.get_by_left(item.0).cloned())
            .filter(|item| item.is_some())
            .map(|item| item.unwrap())
    }
    pub fn generate_endpoint_uuid(endpoint: &Endpoint) -> Uuid {
        ConnectionTracker::generate_uuid(endpoint.ip, endpoint.port)
    }
    pub fn generate_uuid(ip: IpAddr, port: u16) -> Uuid {
        let ip = match ip {
            IpAddr::V4(v4) => v4.to_ipv6_mapped(),
            IpAddr::V6(v6) => v6,
        };

        let hash_str = format!("{}:{}", ip.to_string(), port.to_string());
        let hash_digest = md5::compute(hash_str);

        Uuid::from_bytes(hash_digest.0)
    }
}
