use std::{f32::consts::E, net::IpAddr};

use bimap::BiHashMap;
use gns::{GnsConnection, GnsConnectionInfo};
use uuid::Uuid;

pub trait ToEndpoint {
    fn to_endpoint(&self) -> Endpoint;
}
impl ToEndpoint for GnsConnectionInfo {
    fn to_endpoint(&self) -> Endpoint {
        Endpoint{
            ip: IpAddr::V6(self.remote_address()),
            port: self.remote_port()
        }
    }
}

#[derive(Debug)]
pub struct Endpoint {
    ip: IpAddr,
    port: u16
}

#[derive(Default,Debug)]
pub struct ConnectionTracker {
    connections: BiHashMap<Uuid, GnsConnection>,
}

impl ConnectionTracker {
    pub fn active_players(&self) -> Vec<Uuid> {
        Vec::new()
    }

    pub fn player_connection(&self, player: &Uuid) -> Option<GnsConnection> {
        self.connections
            .get_by_left(player)
            .map(|conn| conn.clone())
    }
    
    pub fn track_player_dicsonnected(&mut self, uuid: &Uuid) {
        if self.connections.contains_left(uuid){
            self.connections.remove_by_left(uuid);
        }
    }
    
    pub fn track_player_connected(&mut self, uuid: Uuid, connection: GnsConnection){
        self.connections.insert(uuid, connection);
    }
    
    pub fn player_by_connection(&self, connection: &GnsConnection) -> Option<&Uuid>{
        self.connections.get_by_right(connection)
    }
    pub fn active_connections(&self) ->impl Iterator<Item = GnsConnection> + '_{
        let connections = &self.connections;
        connections.into_iter().map(|item| item.1.clone()).into_iter()
    }
    pub fn generate_endpoint_uuid(endpoint: &Endpoint) -> Uuid{
        ConnectionTracker::generate_uuid(endpoint.ip,endpoint.port)
    }
    pub fn generate_uuid(ip: IpAddr,port:u16) -> Uuid {
        let ip = match ip {
            IpAddr::V4(v4) => v4.to_ipv6_mapped(),
            IpAddr::V6(v6) => v6,
        };

        let hash_str = format!(
            "{}:{}",
            ip.to_string(),
            port.to_string()
        );
        let hash_digest = md5::compute(hash_str);

        Uuid::from_bytes(hash_digest.0)
    }
}
