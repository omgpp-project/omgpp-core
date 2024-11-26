use std::{
    cell::Cell, io::Read, net::{IpAddr, Ipv4Addr}, rc::Rc, sync::mpsc, thread, time::Instant
};

use client::Client;
use omgpp_core::ConnectionState;
use server::Server;
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        panic!("Provide command line arguments. 1 - to start server, 2 - to start client")
    }
    let start_type = &args[1];
    match &start_type[..] {
        "1" => start_server(),
        "2" => start_client(),
        _ => panic!("error: invalid command"),
    }
}

/*
    sequenceDiagram
    engine->>+client: RunFrame()
    client->>+ server: ReceiveNetworkData
        server ->> server:ReceiveMessagesOnPollGroup

    client->>+server: RunFrame()
        server->>server:SteamGameServer_RunCallbacks
        server->>server:SendDataToClients(with fixed FPS)

    client->>-engine:
*/
fn start_server() {
    println!("Hello! Im Server");
    let mut server = Server::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 55655).unwrap();
    server.register_on_connect_requested(|_server,_id, _endpoint| true);
    server.register_on_connection_state_changed(|server,id, endpoint, state| {
        let msg= format!("Client {:?} {:?}",endpoint,state);
        let status  = server.broadcast(0,msg.as_bytes());

        println!("{:?} {:?} {:?} {:?}", id, state,msg, status)
    });
    server.register_on_message(|ser,id, _endpoint,msg_type, data| {
        _ =ser.broadcast(msg_type,data.as_slice());
     println!(
            "Message from: {:?} Type: {:?} Data: {:?}",
            id, msg_type, data
        );
    });

    let mut prev_time = Instant::now();
    let mut i: i64 = 0;
    loop {
        _ = server.process::<128>();
        let now = Instant::now();
        let delta = now - prev_time;
        if delta.as_millis() > 1000 {
            prev_time = now;
            i += 1;
            //_ = server.broadcast(i, format!("Time is {:?}", now).as_bytes());
        }
        // send data to clients with fixed FPS
    }
}
fn start_client() {
    println!("Hello! Im a client");
    let (tx_channel, rx_channel) = mpsc::channel();
    let _client_connection_thread = thread::spawn(move || {
        let port: u16 = 55655;
        let should_reconnected = Rc::from(Cell::from(false));
        let should_reconnected_cloned = should_reconnected.clone(); // Don't know how to pass it inside a closure without cloning

        let mut client = Client::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

        client.register_on_connection_state_changed(move |endpoint, state| {
            println!("{:?} {:?}", endpoint, state);
            if state == ConnectionState::Disconnected {
                should_reconnected_cloned.set(true);
            }
        });

        client.register_on_message(|endpoint, msg_type, data| {
            println!(
                "Server says: {:?} Type: {:?} Data: {:?}",
                endpoint,
                msg_type,
                String::from_utf8(data)
            );
        });
        client.register_on_rpc(|endpoint, reliable, method_id, request_id, arg_type, data: Vec<u8>|{
            println!(
                "Rpc call {:?} reliable: {:?} method: {:?} request: {:?} arg: {:?} data_size: {:?} data_cap: {:?} data: {:?}",
                endpoint,
                reliable,
                method_id,
                request_id,
                arg_type,
                data.len(),
                data.capacity(),
                String::from_utf8(data)
            ); 
        });
        let _connection_result = client.connect().unwrap();
        let mut sent_count = 0;

        let mut last_update = Instant::now();
        let mut msg_buf = Vec::<String>::new();
        loop {
            if should_reconnected.get() == true {
                should_reconnected.set(false);
                client.connect().unwrap();
            }
            client.process::<128>().unwrap(); // triggers registered callbacks, should be called as freequently as possible
            let since_last_update = Instant::now() - last_update;
            if since_last_update.as_millis() > 2000 {
                last_update = Instant::now();
                if msg_buf.len() > 0 {
                    // take last messages and send
                    for msg in &msg_buf {
                        println!("Sent {}", msg);
                        let data_to_send = if sent_count % 4 == 0 {
                            Some(msg.as_bytes())
                        } else {
                            None
                        };
                        if sent_count % 2 == 0 {
                            _ = client.call_rpc(
                                true,
                                sent_count,
                                (sent_count + 1000) as u64,
                                777,
                                data_to_send,
                            );
                        } else {
                            _ = client.send(777, msg.as_bytes());
                        }
                        sent_count += 1;
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

    // client input
    loop {
        let mut input = String::new();
        _ = std::io::stdin().read_line(&mut input).expect("Some error");
        tx_channel.send(input).unwrap();
    }
}
