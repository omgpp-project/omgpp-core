use std::{cell::RefCell, collections::HashMap, net::IpAddr, rc::Rc};

use crate::chat_message::ChatMessage;
use crate::chat_user::ChatUser;
use server::Server;
use uuid::Uuid;

pub fn start_server(ip: IpAddr, port: u16) {
    println!("Hey! I am a server! Hmmm...");
    let server = Server::new(ip, port).unwrap();

    let users: Rc<RefCell<HashMap<Uuid, ChatUser>>> = Rc::new(RefCell::new(HashMap::new()));

    server.register_on_connect_requested({
        let shared_users = Rc::clone(&users);

        move |_server, id, _endpoint| {
            shared_users
                .borrow_mut()
                .insert(id.clone(), ChatUser::new(id.clone()));

            return true;
        }
    });

    server.register_on_connection_state_changed({
        let shared_users = Rc::clone(&users);

        move |_server, id, _endpoint, state| {
            let mut user_mut = shared_users.borrow_mut();

            let user = user_mut.get_mut(&id);

            if let Some(u) = user {
                u.set_state(state);
            }
        }
    });

    server.register_on_message({
        let shared_users = Rc::clone(&users);

        move |server, id, _endpoint, _msg_type, data| {
            _ = {
                let mut user_mut = shared_users.borrow_mut();

                let user = user_mut.get_mut(&id);

                if let Some(u) = user {
                    let nick = u.get_nick().unwrap_or_else(|| u.get_id().to_string());
                    let chat_msg = ChatMessage::new(nick, String::from_utf8(data).unwrap());

                    let serialized_msg = serde_json::to_string(&chat_msg).unwrap();
                    _ = server.broadcast(0, serialized_msg.as_bytes())
                }
            }
        }
    });

    loop {
        _ = server.process::<128>();
    }
}
