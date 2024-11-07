use bimap::BiHashMap;
use gns::GnsConnection;
use uuid::Uuid;

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
}
