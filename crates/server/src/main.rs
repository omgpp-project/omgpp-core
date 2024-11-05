use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::mpsc,
    thread,
    time::Instant,
};

use gns::{GnsGlobal, GnsSocket, GnsUtils, IsCreated};
use gns_sys::k_nSteamNetworkingSend_Reliable;
use gns_sys::k_nSteamNetworkingSend_Unreliable;
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

fn start_server() {
    println!("Hello! Im Server");
    let mut server = Server::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 55655).unwrap();
    server.register_on_connect_requested(|id| true);
    server.register_on_connection_state_changed(|id, state| println!("{:?} {:?}", id, state));
    server.register_on_message(|id, _msg_type,data| {
        println!("{:?} {:?}", id, data);
    });

    loop {
        _ = server.process::<128>();
    }

    // Now that we initiated a connection, there is three operation we must loop over:
    // - polling for new messages
    // - polling for connection status change
    // - polling for callbacks (low-level callbacks required by the underlying library).
    // Important to know, regardless of the type of socket, whether it is in [`IsClient`] or [`IsServer`] state, theses three operations are the same.
    // The only difference is that polling for messages and status on the client only act on the client connection, while polling for messages and status on a server yield event for all connected clients.

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
    // loop {
    //     // Run the low-level callbacks.
    //     server.poll_callbacks();

    //     let _actual_nb_of_messages_processed = server.poll_messages::<128>(|message| {
    //         println!("Msg income");
    //         println!("{}", core::str::from_utf8(message.payload()).unwrap());
    //     });

    //     // Don't do anything with events.
    //     // One would check the event for connection status, i.e. doing something when we are connected/disconnected from the server.
    //     let _actual_nb_of_events_processed = server.poll_event::<128>(|event| {

    //         match (event.old_state(), event.info().state()) {
    //             // A client is about to connect, accept it.
    //             (
    //               ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_None,
    //               ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting,
    //             ) => {
    //               let _result = server.accept(event.connection());
    //             }
    //             _=>()
    //         }
    //         println!(
    //             "Connection {}",
    //             format!("{:?} {:?}", &event.info().state(), event.info().remote_address())
    //         );
    //     });

    // send data to users with fixed FPS
    // }
}
fn start_client() {
    println!("Hello! Im a client");
    let (tx_channel, rx_channel) = mpsc::channel();
    let _client_connection_thread = thread::spawn(move || {
        let gns_global = GnsGlobal::get().unwrap();
        let gns_utils = GnsUtils::new().unwrap();

        let port: u16 = 55655;
        let gns_socket = GnsSocket::<IsCreated>::new(&gns_global, &gns_utils).unwrap();
        let client = gns_socket.connect(Ipv4Addr::LOCALHOST.to_ipv6_mapped(), port).unwrap();
        let mut last_update = Instant::now();
        let mut msg_buf = Vec::<String>::new();
        loop {
            client.poll_callbacks();

            let _actual_nb_of_messages_processed = client.poll_messages::<128>(|message| {
                println!("{}", core::str::from_utf8(message.payload()).unwrap());
            });

            let _actual_nb_of_events_processed = client.poll_event::<128>(|event| {
                let conn = event.connection();
                println!(
                    "Connection Client {}",
                    format!("{:?} {:?}", conn, event.info().remote_address())
                );
            });
            let since_last_update = Instant::now() - last_update;
            if since_last_update.as_millis() > 2000 {
                last_update = Instant::now();
                if msg_buf.len() > 0 {
                    // take last messages and send
                    for msg in &msg_buf {
                        println!("Sent {}", msg);
                        client.send_messages(vec![client.utils().allocate_message(
                            client.connection(),
                            k_nSteamNetworkingSend_Unreliable,
                            msg.as_bytes(),
                        )]);
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
        tx_channel.send(input).unwrap();
    }
}
