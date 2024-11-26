use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub nick: String,
    pub text: String,
    pub created_at: SystemTime,
}

impl ChatMessage {
    pub fn new(nick: String, text: String) -> ChatMessage {
        let now = SystemTime::now();

        ChatMessage {
            nick,
            text,
            created_at: now,
        }
    }
}
