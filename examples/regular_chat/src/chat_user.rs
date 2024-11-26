use omgpp_core::ConnectionState;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChatUser {
    id: Uuid,
    nick: Option<String>,
    state: ConnectionState,
}

impl ChatUser {
    pub fn new(id: Uuid) -> ChatUser {
        ChatUser {
            id,
            nick: None,
            state: ConnectionState::None,
        }
    }

    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state
    }

    pub fn get_id(&self) -> Uuid {
        return self.id;
    }

    pub fn get_nick(&self) -> Option<String> {
        return self.nick.clone();
    }
}
