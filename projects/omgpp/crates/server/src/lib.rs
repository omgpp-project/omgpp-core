pub mod connection_tracker;
pub mod ffi;

use std::cell::{RefCell, RefMut};
use std::{fmt::Debug, marker::PhantomData, net::IpAddr};

use connection_tracker::ConnectionTracker;
use gns::ToReceive;
use gns::{GnsConnectionEvent, GnsNetworkMessage, GnsSocket, IsCreated, IsServer};
use gns_sys::{
    k_nSteamNetworkingSend_Reliable, k_nSteamNetworkingSend_Unreliable,
    ESteamNetworkingConnectionState,
};
use omgpp_core::ToEndpoint;
use omgpp_core::{
    messages::general_message::GeneralOmgppMessage, ConnectionState, Endpoint, TransmitterHelper,
    GNS,
};
use protobuf::Message;
use uuid::Uuid;

type OnConnectRequestCallback = Box<dyn Fn(&Server, &Uuid, &Endpoint) -> bool + 'static>;
type OnConnectionChangedCallback = Box<dyn Fn(&Server, &Uuid, &Endpoint, ConnectionState) + 'static>;
type OnMessageCallback = Box<dyn Fn(&Server, &Uuid, &Endpoint, i64, Vec<u8>) + 'static>;

type ServerResult<T> = Result<T, String>; // TODO replace error with enum

struct ServerCallbacks {
    on_connect_requested_callback: OnConnectRequestCallback,
    on_connection_changed_callback: Option<OnConnectionChangedCallback>,
    on_message_callback: Option<OnMessageCallback>,
}
pub struct Server<'a> {
    ip: IpAddr,
    port: u16,
    connection_tracker: RefCell<ConnectionTracker>,
    socket: GnsSocket<'static, 'static, IsServer>,
    callbacks: RefCell<ServerCallbacks>,
    phantom: PhantomData<&'a bool>,
}

impl<'a> Server<'a> {
    pub fn new(ip: IpAddr, port: u16) -> ServerResult<Server<'a>> {
        let gns = GNS.as_ref()?;
        let gns_socket = GnsSocket::<IsCreated>::new(&gns.global, &gns.utils).unwrap();
        let address_to_bind = match ip {
            IpAddr::V4(v4) => v4.to_ipv6_mapped(),
            IpAddr::V6(v6) => v6,
        };
        let server_socket = gns_socket
            .listen(address_to_bind, port)
            .or(ServerResult::Err("Cannot create server socket".to_string()))?;
        Ok(Server {
            ip,
            port,
            socket: server_socket,
            connection_tracker: Default::default(),
            callbacks: RefCell::new(ServerCallbacks {
                on_connect_requested_callback: Box::new(|_server, _id, _endpoint| true),
                on_connection_changed_callback: None,
                on_message_callback: None,
            }),
            phantom: Default::default(),
        })
    }
    // TODO Maybe it worth to return a Iterator instead of cloning
    pub fn active_players(&self) -> Vec<(Uuid, Endpoint)> {
        self.connection_tracker.borrow().active_players()
    }
    pub fn socket(&self) -> &GnsSocket<'static, 'static, IsServer> {
        &self.socket
    }
    /// Make 1 server cycle.
    /// Generic paramter N specfies maximum number of events and messages to process per a call
    pub fn process<const N: usize>(&self) -> ServerResult<()> {
        let socket = &self.socket;
        socket.poll_callbacks();
        let mut socket_op_result = ServerResult::Ok(());
        let _processed_event_count = socket.poll_event::<N>(|event| {
            socket_op_result = Server::process_connection_events(
                self,
                event,
                &self.socket,
                &self.callbacks.borrow(),
                &self.connection_tracker,
            )
        });

        let _processed_msg_count = socket.poll_messages::<N>(|msg| {
            socket_op_result = Server::process_messages(
                self,
                msg,
                &self.connection_tracker.borrow(),
                &self.callbacks.borrow(),
            )
        });

        socket_op_result
    }
    pub fn socket_poll_messages(&mut self, socket: &GnsNetworkMessage<ToReceive>) {}
    pub fn send(&self, player: &Uuid, msg_type: i64, data: &[u8]) -> ServerResult<()> {
        self.send_with_flags(player, msg_type, data, k_nSteamNetworkingSend_Unreliable)
    }

    pub fn send_reliable(&self, player: &Uuid, msg_type: i64, data: &[u8]) -> ServerResult<()> {
        self.send_with_flags(player, msg_type, data, k_nSteamNetworkingSend_Reliable)
    }

    pub fn broadcast(&self, msg_type: i64, data: &[u8]) -> ServerResult<()> {
        let msg_bytes = Server::create_general_message(msg_type, data)
            .or_else(|_or| Err("Cannot create general message".to_string()))?;

        self.broadcast_with_flags(k_nSteamNetworkingSend_Unreliable, msg_bytes.as_slice())
    }
    pub fn broadcast_reliable(&self, msg_type: i64, data: &[u8]) -> ServerResult<()> {
        let msg_bytes = Server::create_general_message(msg_type, data)
            .or_else(|_or| Err("Cannot create general message".to_string()))?;
        self.broadcast_with_flags(k_nSteamNetworkingSend_Reliable, msg_bytes.as_slice())
    }

    pub fn register_on_connect_requested(
        &self,
        callback: impl Fn(&Server,&Uuid, &Endpoint) -> bool + 'static,
    ) {
        self.callbacks.borrow_mut().on_connect_requested_callback = Box::from(callback);
    }
    pub fn register_on_connection_state_changed(
        &self,
        callback: impl Fn(&Server,&Uuid, &Endpoint, ConnectionState) + 'static,
    ) {
        self.callbacks.borrow_mut().on_connection_changed_callback = Some(Box::from(callback));
    }
    pub fn register_on_message(
        &self,
        callback: impl Fn(&Server, &Uuid, &Endpoint, i64, Vec<u8>) + 'static,
    ) {
        self.callbacks.borrow_mut().on_message_callback = Some(Box::from(callback));
    }

    fn process_connection_events(
        &self,
        event: GnsConnectionEvent,
        socket: &GnsSocket<IsServer>,
        callbacks: &ServerCallbacks,
        connection_tracker: &RefCell<ConnectionTracker>,
    ) -> ServerResult<()> {
        let endpoint = event.info().to_endpoint();
        let player_uuid = ConnectionTracker::generate_endpoint_uuid(&endpoint);
        match (event.old_state(), event.info().state()) {
            // player tries to connect
            (
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_None,
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting,
            ) => {
                if let Some(cb) = &callbacks.on_connection_changed_callback{
                    cb(self,&player_uuid, &endpoint, ConnectionState::Connecting);      // TODO add host and port as parameters
                }
                let should_accept = (callbacks.on_connect_requested_callback)(self,&player_uuid,&endpoint);
                if should_accept {
                    socket.accept(event.connection()).or_else(|_err| {
                        ServerResult::Err("Cannot accept the connection".to_string())
                    })?;
                } else {
                    // watch all possible reasons in ESteamNetConnectionEnd at steamworks_sdk_160\sdk\public\steam\steamnetworkingtypes.h (SteamworksSDK)
                    socket.close_connection(
                        event.connection(),
                        0,      // k_ESteamNetConnectionEnd_Invalid 
                        "You are not allowed to connect",
                        false,
                    );
                }
            }
            // player disconnected gracefully (? or may be not)
            (
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting
                | ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connected,
                 ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_ClosedByPeer
                |ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_ProblemDetectedLocally,
            ) => {
                connection_tracker.borrow_mut().track_player_disconnected(&player_uuid);

                if let Some(cb) = &callbacks.on_connection_changed_callback {
                    cb(self,&player_uuid, &endpoint, ConnectionState::Disconnected);
                }
            }
            // player connected
            (
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting,
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connected,
            ) => {
                connection_tracker.borrow_mut().track_player_connected(player_uuid.clone(),endpoint, event.connection());

                if let Some(cb) = &callbacks.on_connection_changed_callback {
                    cb(self,&player_uuid, &endpoint, ConnectionState::Connected);
                }
            }

            (_, _) => (),
        }
        Ok(())
    }

    fn process_messages(
        &self,
        event: &GnsNetworkMessage<ToReceive>,
        connection_tracker: &ConnectionTracker,
        callbacks: &ServerCallbacks,
    ) -> ServerResult<()> {
        let data = event.payload();
        let connection = event.connection();
        let sender = connection_tracker
            .player_by_connection(&connection)
            .ok_or_else(|| "Unknown connection".to_string())?;
        let endpoint = connection_tracker
            .player_endpoint(sender)
            .ok_or_else(|| "Unknown endpoint".to_string())?;

        // cb stands for callback
        match &callbacks.on_message_callback {
            // we have callback
            Some(cb) => match GeneralOmgppMessage::parse_from_bytes(data).ok() {
                // we decoded message
                Some(msg) => cb(self, sender, endpoint, msg.type_, Vec::from(msg.data)),
                _ => println!("Cannot decode message"),
            },
            _ => {}
        }
        Ok(())
    }

    fn send_with_flags(
        &self,
        player: &Uuid,
        msg_type: i64,
        data: &[u8],
        flags: i32,
    ) -> ServerResult<()> {
        let connection = self
            .connection_tracker
            .borrow()
            .player_connection(player)
            .ok_or_else(|| "There is not such player to send")?;

        let msg_bytes = Server::create_general_message(msg_type, data)
            .or_else(|_or| Err("Cannot create general message".to_string()))?;

        // TODO check send result
        let _send_result =
            TransmitterHelper::send(&self.socket, &[connection], flags, msg_bytes.as_slice());
        Ok(())
    }
    fn broadcast_with_flags(&self, flags: i32, data: &[u8]) -> ServerResult<()> {
        let tracker = self.connection_tracker.borrow();
        let connections = tracker.active_connections();
        let _res = TransmitterHelper::send_with_iter(&self.socket, connections, flags, data);
        Ok(())
    }

    fn create_general_message(msg_type: i64, data: &[u8]) -> protobuf::Result<Vec<u8>> {
        let mut msg = GeneralOmgppMessage::new();
        msg.type_ = msg_type;
        msg.data = Vec::from(data);
        let bytes = msg.write_to_bytes()?;
        return Ok(bytes);
    }
}

impl<'a> Debug for Server<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("ip", &self.ip)
            .field("port", &self.port)
            .field("connection_tracker", &self.connection_tracker)
            .finish()
    }
}
