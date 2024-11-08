pub mod connection_tracker;
use std::{fmt::Debug, marker::PhantomData, net::IpAddr, sync::LazyLock};

use connection_tracker::{ConnectionTracker, Endpoint, ToEndpoint};
use either::Either;
use gns::{
    GnsConnection, GnsConnectionEvent, GnsDroppable, GnsGlobal, GnsNetworkMessage, GnsSocket,
    GnsUtils, IsCreated, IsReady, IsServer, ToReceive,
};
use gns_sys::{
    k_nSteamNetworkingSend_Reliable, k_nSteamNetworkingSend_Unreliable,
    ESteamNetworkingConnectionState,
};
use omgpp_core::messages::general_message::GeneralOmgppMessage;
use protobuf::Message;
use uuid::Uuid;

type OnConnectRequestCallback = Box<dyn Fn(&Uuid, &Endpoint) -> bool + Send + 'static>;
type OnConnectionChangedCallback = Box<dyn Fn(&Uuid, &Endpoint, ConnectionState) + Send + 'static>;
type OnMessageCallback = Box<dyn Fn(&Uuid, i64, Vec<u8>) + Send + 'static>;

type ServerResult<T> = Result<T, String>; // TODO replace error with enum

struct GnsWrapper {
    global: GnsGlobal,
    utils: GnsUtils,
}
unsafe impl Send for GnsWrapper {}
unsafe impl Sync for GnsWrapper {}

static GNS: LazyLock<ServerResult<GnsWrapper>> = LazyLock::new(|| {
    Ok(GnsWrapper {
        global: GnsGlobal::get()?,
        utils: GnsUtils::new().ok_or("Error occurred when creating GnsUtils")?,
    })
});

#[allow(dead_code)]
pub struct TransmitterHelper {}

impl TransmitterHelper {
    pub fn send<T: GnsDroppable + IsReady>(
        socket: &GnsSocket<'_, '_, T>,
        connections: &[GnsConnection],
        flags: i32,
        data: &[u8],
    ) -> Vec<Either<u64, gns_sys::EResult>> {
        TransmitterHelper::send_with_iter(
            socket,
            connections.into_iter().map(|i| i.to_owned()),
            flags,
            data,
        )
    }
    pub fn send_with_iter<T: GnsDroppable + IsReady>(
        socket: &GnsSocket<'_, '_, T>,
        connections: impl Iterator<Item = GnsConnection>,
        flags: i32,
        data: &[u8],
    ) -> Vec<Either<u64, gns_sys::EResult>> {
        let messages = connections
            .map(|connection| {
                socket
                    .utils()
                    .allocate_message(connection.clone(), flags, data)
            })
            .collect::<Vec<_>>();

        match messages.len() > 0 {
            true => socket.send_messages(messages),
            false => vec![],
        }
        /*
            if res.get(0).unwrap().is_right() {
                return ServerResult::Err("Some error occured when sending the message".to_string());
            }
        */
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConnectionState {
    Disconnected = 0,
    Disconnecting = 1,
    Connecting = 2,
    Connected = 3,
}
struct ServerCallbacks {
    on_connect_requested_callback: OnConnectRequestCallback,
    on_connection_changed_callback: Option<OnConnectionChangedCallback>,
    on_message_callback: Option<OnMessageCallback>,
}

pub struct Server<'a> {
    ip: IpAddr,
    port: u16,
    connection_tracker: ConnectionTracker,
    socket: GnsSocket<'static, 'static, IsServer>,
    callbacks: ServerCallbacks,
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
            callbacks: ServerCallbacks {
                on_connect_requested_callback: Box::new(|_id, _endpoint| true),
                on_connection_changed_callback: None,
                on_message_callback: None,
            },
            phantom: Default::default(),
        })
    }
    // TODO Maybe it worth to return a Iterator instead of cloning
    pub fn active_players(&self) -> Vec<(Uuid, Endpoint)> {
        self.connection_tracker.active_players()
    }
    pub fn socket(&self) -> &GnsSocket<'static,'static,IsServer>{
        &self.socket
    }
    /// Make 1 server cycle.
    /// Generic paramter N specfies maximum number of events and messages to process per a call
    pub fn process<const N: usize>(&mut self) -> ServerResult<()> {
        let socket = &self.socket;
        socket.poll_callbacks();
        let mut socket_op_result = ServerResult::Ok(());
        let _processed_event_count = socket.poll_event::<N>(|event| {
            socket_op_result = Server::process_connection_events(
                event,
                &self.socket,
                &self.callbacks,
                &mut self.connection_tracker,
            )
        });
        let _processed_msg_count = socket.poll_messages::<N>(|msg| {
            socket_op_result =
                Server::process_messages(msg, &self.connection_tracker, &self.callbacks)
        });

        socket_op_result
    }

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
        &mut self,
        callback: impl Fn(&Uuid, &Endpoint) -> bool + 'static + Send,
    ) {
        self.callbacks.on_connect_requested_callback = Box::from(callback);
    }
    pub fn register_on_connection_state_changed(
        &mut self,
        callback: impl Fn(&Uuid, &Endpoint, ConnectionState) + 'static + Send,
    ) {
        self.callbacks.on_connection_changed_callback = Some(Box::from(callback));
    }
    pub fn register_on_message(&mut self, callback: impl Fn(&Uuid, i64, Vec<u8>) + 'static + Send) {
        self.callbacks.on_message_callback = Some(Box::from(callback));
    }

    fn process_connection_events(
        event: GnsConnectionEvent,
        socket: &GnsSocket<IsServer>,
        callbacks: &ServerCallbacks,
        connection_tracker: &mut ConnectionTracker,
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
                    cb(&player_uuid, &endpoint, ConnectionState::Connecting);      // TODO add host and port as parameters
                }
                let should_accept = (callbacks.on_connect_requested_callback)(&player_uuid,&endpoint);
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
                connection_tracker.track_player_disconnected(&player_uuid);

                if let Some(cb) = &callbacks.on_connection_changed_callback {
                    cb(&player_uuid, &endpoint, ConnectionState::Disconnected);
                }
            }
            // player connected
            (
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting,
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connected,
            ) => {
                connection_tracker.track_player_connected(player_uuid.clone(),endpoint, event.connection());

                if let Some(cb) = &callbacks.on_connection_changed_callback {
                    cb(&player_uuid, &endpoint, ConnectionState::Connected);
                }
            }

            (_, _) => (),
        }
        Ok(())
    }

    fn process_messages(
        event: &GnsNetworkMessage<ToReceive>,
        connection_tracker: &ConnectionTracker,
        callbacks: &ServerCallbacks,
    ) -> ServerResult<()> {
        let data = event.payload();
        let connection = event.connection();
        let sender = connection_tracker
            .player_by_connection(&connection)
            .ok_or_else(|| "Unknown connection".to_string())?;
        // cb stands for callback
        match &callbacks.on_message_callback {
            // we have callback
            Some(cb) => match GeneralOmgppMessage::parse_from_bytes(data).ok() {
                // we decoded message
                Some(msg) => cb(sender, msg.type_, Vec::from(msg.data)),
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
        let connections = self.connection_tracker.active_connections();
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
