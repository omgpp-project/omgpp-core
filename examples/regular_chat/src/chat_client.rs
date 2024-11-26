use std::{cell::Cell, net::IpAddr, rc::Rc, sync::mpsc, thread, time::Instant};

use crate::chat_message::ChatMessage;
use client::Client;
use omgpp_core::ConnectionState;

pub fn start_client(ip: IpAddr, port: u16) {
    let (tx_channel, rx_channel) = mpsc::channel();

    let _client_connection_thread = thread::spawn(move || {
        let should_reconnected = Rc::from(Cell::from(false));

        // Don't know how to pass it inside a closure without cloning
        let should_reconnected_cloned = should_reconnected.clone();

        let mut client = Client::new(ip, port);

        client.register_on_connection_state_changed(move |endpoint, state| {
            println!("{:?} {:?}", endpoint, state);
            if state == ConnectionState::Disconnected {
                should_reconnected_cloned.set(true);
            }
        });

        client.register_on_message(|_endpoint, _msg_type, data| {
            let json_data = String::from_utf8(data).unwrap_or("{}".to_string());

            let chat_message: Option<ChatMessage> =
                serde_json::from_str::<ChatMessage>(&json_data).ok();

            if let Some(ch_m) = chat_message {
                println!(">>> {:?} {:?}: {:?}", ch_m.created_at, ch_m.nick, ch_m.text)
            } else {
                println!("Всё очень плохо!!!")
            }
        });
        let _connection_result = client.connect().unwrap();

        let mut last_update = Instant::now();
        let mut msg_buf = Vec::<String>::new();
        loop {
            if should_reconnected.get() == true {
                should_reconnected.set(false);
                client.connect().unwrap();
            }

            // triggers registered callbacks, should be called as freequently as possible
            client.process::<128>().unwrap();
            let since_last_update = Instant::now() - last_update;
            if since_last_update.as_millis() > 2000 {
                last_update = Instant::now();
                if msg_buf.len() > 0 {
                    // take last messages and send
                    for msg in &msg_buf {
                        _ = client.send(777, msg.as_bytes());
                    }
                    msg_buf.clear();
                }
            }
            loop {
                if let Ok(received) = rx_channel.try_recv() {
                    msg_buf.push(received);
                } else {
                    break;
                }
            }
        }
    });

    // user input
    loop {
        let mut input = String::new();
        _ = std::io::stdin().read_line(&mut input).expect("Some error");
        input = input.trim().to_string();
        print!("\x1B[1A"); // Clear the current line
        print!("           "); // Clear the current line
        tx_channel.send(input).unwrap();
    }
}
