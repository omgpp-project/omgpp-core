mod ffi;

use std::net::IpAddr;

use gns::{GnsSocket, IsClient, IsCreated};
use gns_sys::{
    k_nSteamNetworkingSend_Reliable, k_nSteamNetworkingSend_Unreliable,
    ESteamNetworkingConnectionState,
};
use omgpp_core::{
    messages::general_message::{
        general_omgpp_message::{self, Data},
        GeneralOmgppMessage,
    },
    ConnectionState, Endpoint, ToEndpoint, TransmitterHelper, GNS,
};
use protobuf::Message;

type OnConnectionChangedCallback = Box<dyn Fn(&Endpoint, ConnectionState) + 'static>;
type OnMessageCallback = Box<dyn Fn(&Endpoint, i64, Vec<u8>) + 'static>;
type OnRpcCallback = Box<dyn Fn(&Endpoint, bool, i64, u64, i64, Vec<u8>) + 'static>;

type ClientResult<T> = Result<T, String>; // TODO replace error with enum
struct ClientCallbacks {
    on_connection_changed_callback: Option<OnConnectionChangedCallback>,
    on_message_callback: Option<OnMessageCallback>,
    on_rpc_callback: Option<OnRpcCallback>,
}
//TODO In order to support multiple servers, track multiple GnsSockets
struct ConnectionTracker {
    server_endpoint: Endpoint,
    state: ConnectionState,
}
impl ConnectionTracker {
    fn track_connection_state(&mut self, state: ConnectionState) {
        self.state = state;
    }
}
// TODO In order to support multiple servers, move `socket` in ConnectionTracker
pub struct Client {
    socket: Option<GnsSocket<'static, 'static, IsClient>>,
    callbacks: ClientCallbacks,
    connection_tracker: ConnectionTracker,
}
impl Client {
    pub fn new(server_ip: IpAddr, server_port: u16) -> Client {
        Client {
            socket: None,
            callbacks: ClientCallbacks {
                on_connection_changed_callback: None,
                on_message_callback: None,
                on_rpc_callback: None,
            },
            connection_tracker: ConnectionTracker {
                state: ConnectionState::None,
                server_endpoint: Endpoint {
                    ip: server_ip,
                    port: server_port,
                },
            },
        }
    }
    pub fn register_on_connection_state_changed(
        &mut self,
        callback: impl Fn(&Endpoint, ConnectionState) + 'static,
    ) {
        self.callbacks.on_connection_changed_callback = Some(Box::from(callback));
    }
    pub fn register_on_message(&mut self, callback: impl Fn(&Endpoint, i64, Vec<u8>) + 'static) {
        self.callbacks.on_message_callback = Some(Box::from(callback));
    }
    pub fn register_on_rpc(
        &mut self,
        callback: impl Fn(&Endpoint, bool, i64, u64, i64, Vec<u8>) + 'static,
    ) {
        self.callbacks.on_rpc_callback = Some(Box::from(callback));
    }
    pub fn connect(&mut self) -> ClientResult<()> {
        let old_socket = &self.socket;
        let current_connection_state = &self.connection_tracker.state;

        match (old_socket, current_connection_state) {
            (Some(_), ConnectionState::Connecting | ConnectionState::Connected) => {
                Err("Already connected to server")?
            }
            _ => (),
        }
        let gns = GNS.as_ref()?;
        let gns_socket = GnsSocket::<IsCreated>::new(&gns.global, &gns.utils).unwrap();

        let address_to_connect = match self.connection_tracker.server_endpoint.ip {
            IpAddr::V4(v4) => v4.to_ipv6_mapped(),
            IpAddr::V6(v6) => v6,
        };
        let port = self.connection_tracker.server_endpoint.port;
        let client_socket = gns_socket
            .connect(address_to_connect, port)
            .or(Err("Cannot create socket to connect to server".to_string()))?;

        self.socket = Some(client_socket);
        Ok(())
    }

    pub fn disconnect(&self) {
        if let Some(socket) = &self.socket {
            socket.close_connection(socket.connection(), 0, "", false);
        }
    }

    pub fn process<const N: usize>(&mut self) -> ClientResult<()> {
        if self.socket.is_none() {
            return Err("Socket not initialized".to_string());
        }
        let socket = self.socket.as_ref().unwrap();
        socket.poll_callbacks();
        let mut socket_op_is_success = ClientResult::Ok(());
        let _processed_event_count = socket.poll_event::<N>(|event| {
            Client::process_connection_events(event, &self.callbacks, &mut self.connection_tracker)
        });
        let _processed_msg_count = socket.poll_messages::<N>(|msg| {
            socket_op_is_success =
                Client::process_messages(msg, &self.connection_tracker, &self.callbacks);
        });
        socket_op_is_success
    }

    pub fn send(&self, msg_type: i64, data: &[u8]) -> ClientResult<()> {
        self.send_with_flags(k_nSteamNetworkingSend_Unreliable, msg_type, data)
    }
    pub fn send_reliable(&self, msg_type: i64, data: &[u8]) -> ClientResult<()> {
        self.send_with_flags(k_nSteamNetworkingSend_Reliable, msg_type, data)
    }

    pub fn call_rpc(
        &self,
        reliable: bool,
        method_id: i64,
        request_id: u64,
        arg_type: i64,
        arg_data: Option<&[u8]>,
    ) -> ClientResult<()> {
        if let Some(socket) = &self.socket {
            let msg_bytes = create_rpc_message(reliable, method_id, request_id, arg_type, arg_data)
                .or_else(|_or| Err("Cannot create rpc message".to_string()))?;

            let flags = match reliable {
                true => k_nSteamNetworkingSend_Reliable,
                false => k_nSteamNetworkingSend_Unreliable,
            };

            // TODO check send result
            let _send_results =
                TransmitterHelper::send(socket, &[socket.connection()], flags, &msg_bytes);
        }
        Ok(())
    }

    fn send_with_flags(&self, flags: i32, msg_type: i64, data: &[u8]) -> ClientResult<()> {
        if let Some(socket) = &self.socket {
            let msg_bytes = create_general_message(msg_type, data)
                .or_else(|_err| Err("Cannot create general message"))?;

            // TODO check send result
            let _send_results =
                TransmitterHelper::send(socket, &[socket.connection()], flags, &msg_bytes);
        }
        Ok(())
    }
    fn process_connection_events(
        event: gns::GnsConnectionEvent,
        callbacks: &ClientCallbacks,
        connection_tracker: &mut ConnectionTracker,
    ) {
        let endpoint = event.info().to_endpoint();
        match (event.old_state(), event.info().state()) {
            // player tries to connect
            (
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_None,
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting,
            ) => {
                connection_tracker.track_connection_state(ConnectionState::Connecting);
                if let Some(cb) = &callbacks.on_connection_changed_callback{
                    cb(&endpoint, ConnectionState::Connecting);      // TODO add host and port as parameters
                }
            }
            // player disconnected gracefully (? or may be not)
            (
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting
                | ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connected,
                 ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_ClosedByPeer
                |ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_ProblemDetectedLocally,
            ) => {
                connection_tracker.track_connection_state(ConnectionState::Disconnected);
                if let Some(cb) = &callbacks.on_connection_changed_callback {
                    cb(&endpoint, ConnectionState::Disconnected);
                }
            }
            // player connected
            (
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting,
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connected,
            ) => {
                connection_tracker.track_connection_state(ConnectionState::Connected);
                if let Some(cb) = &callbacks.on_connection_changed_callback {
                    cb(&endpoint, ConnectionState::Connected);
                }
            }

            (_, _) => (),
        }
    }

    fn process_messages(
        gns_msg: &gns::GnsNetworkMessage<gns::ToReceive>,
        connection_tracker: &ConnectionTracker,
        callbacks: &ClientCallbacks,
    ) -> ClientResult<()> {
        let data = gns_msg.payload();
        let sender = &connection_tracker.server_endpoint;
        if let Some(decoded) = GeneralOmgppMessage::parse_from_bytes(data).ok() {
            // we decoded the message
            match decoded.data {
                Some(Data::Message(message)) => {
                    // cb stands for callback
                    if let Some(cb) = &callbacks.on_message_callback {
                        cb(sender, message.type_, message.data)
                    }
                }
                Some(Data::Rpc(rpc_call)) => {
                    if let Some(rpc_callback) = &callbacks.on_rpc_callback {
                        rpc_callback(
                            sender,
                            rpc_call.reliable,
                            rpc_call.method_id,
                            rpc_call.request_id,
                            rpc_call.arg_type,
                            rpc_call.arg_data,
                        );
                    };
                }
                _ => (),
            }
        } else {
            // cannot decode message;
        }
        Ok(())
    }
}

fn create_general_message(msg_type: i64, data: &[u8]) -> protobuf::Result<Vec<u8>> {
    let mut payload = GeneralOmgppMessage::new();
    let mut message = general_omgpp_message::Message::new();
    message.type_ = msg_type;
    message.data = Vec::from(data); // somehow get rid of unessesary array copying
    payload.data = Some(Data::Message(message));
    let bytes = payload.write_to_bytes()?;
    return Ok(bytes);
}
fn create_rpc_message(
    reliable: bool,
    method_id: i64,
    request_id: u64,
    arg_type: i64,
    data: Option<&[u8]>,
) -> protobuf::Result<Vec<u8>> {
    let mut payload = GeneralOmgppMessage::new();
    let mut rpc = general_omgpp_message::RpcCall::new();
    rpc.reliable = reliable;
    rpc.method_id = method_id;
    rpc.request_id = request_id;
    rpc.arg_type = arg_type;
    rpc.arg_data = match data {
        Some(byte_array) => Vec::from(byte_array),
        None => Vec::new(),
    };
    payload.data = Some(Data::Rpc(rpc));
    let bytes = payload.write_to_bytes()?;
    return Ok(bytes);
}
