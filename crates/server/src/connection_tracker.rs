use std::net::IpAddr;

use bimap::BiHashMap;
use gns::{GnsConnection};
use omgpp_core::Endpoint;
use uuid::Uuid;


#[derive(Default, Debug)]
pub struct ConnectionTracker {
    connections: BiHashMap<Uuid, GnsConnection>,
    endpoints: BiHashMap<Uuid, Endpoint>,
}

impl ConnectionTracker {
    pub fn active_players(&self) -> Vec<(Uuid, Endpoint)> {
        let endpoints = &self.endpoints;
        let active_endpoints = endpoints
            .into_iter()
            .map(|item| (item.0.clone(), item.1.clone()))
            .collect();
        active_endpoints
    }

    pub fn player_connection(&self, player: &Uuid) -> Option<GnsConnection> {
        self.connections
            .get_by_left(player)
            .map(|conn| conn.clone())
    }

    pub fn track_player_disconnected(&mut self, uuid: &Uuid) {
        if self.connections.contains_left(uuid) {
            self.connections.remove_by_left(uuid);
        }
        if self.endpoints.contains_left(uuid){
            self.endpoints.remove_by_left(uuid);
        }
    }

    pub fn track_player_connected(&mut self, uuid: Uuid, endpoint:Endpoint,connection: GnsConnection) {
        self.connections.insert(uuid, connection);
        // TODO decide what todo when we have already associated endpoint
        let _old_endpoint = self.endpoints.insert(uuid, endpoint);   
    }

    pub fn player_by_connection(&self, connection: &GnsConnection) -> Option<&Uuid> {
        self.connections.get_by_right(connection)
    }
    pub fn active_connections(&self) -> impl Iterator<Item = GnsConnection> + '_ {
        let connections = &self.connections;
        connections
            .into_iter()
            .map(|item| item.1.clone())
            .into_iter()
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
