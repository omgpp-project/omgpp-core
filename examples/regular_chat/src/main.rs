use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};

mod chat_user;
mod chat_message;
mod chat_server;
mod chat_client;

use chat_server::start_server;
use chat_client::start_client;


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("Provide command line arguments. 1 - to start server, 2 - to start client")
    }

    let start_type = &args[1];

    let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let port = 55655;

    match &start_type[..] {
        "1" => start_server(ip, port),
        "2" => start_client(ip, port),
        _ => panic!("error: invalid command"),
    }

    println!("Server is started...");
}

