use md5;
use std::{collections::HashMap, fmt::Debug, fs::OpenOptions, net::IpAddr, sync::LazyLock};

use gns::{
    GnsConnection, GnsConnectionEvent, GnsGlobal, GnsNetworkMessage, GnsSocket, GnsUtils,
    IsCreated, IsServer, ToReceive,
};
use gns_sys::ESteamNetworkingConnectionState;
use uuid::Uuid;

type OnConnectRequestCallback = Box<dyn Fn(&Uuid) -> bool + Send + 'static>;
type OnConnectionChangedCallback = Box<dyn Fn(&Uuid, ConnectionState) + Send + 'static>;
type OnMessageCallback = Box<dyn Fn(&Uuid, i32, Vec<u8>) + Send + 'static>;

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
    active_connetions: HashMap<Uuid, &'a GnsConnection>,
    socket: GnsSocket<'static, 'static, IsServer>,
    callbacks: ServerCallbacks,
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
            active_connetions: HashMap::new(),
            callbacks: ServerCallbacks {
                on_connect_requested_callback: Box::new(|_id| true),
                on_connection_changed_callback: None,
                on_message_callback: None,
            },
        })
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
                &mut self.active_connetions,
            )
        });
        let _processed_msg_count =
            socket.poll_messages::<N>(|msg| socket_op_result = self.process_messages(msg));

        socket_op_result
    }
    fn process_connection_events(
        event: GnsConnectionEvent,
        socket: &GnsSocket<IsServer>,
        callbacks: &ServerCallbacks,
        active_connetions: &mut HashMap<Uuid, &'a GnsConnection>,
    ) -> ServerResult<()> {
        let hash_str = format!(
            "{}:{}",
            event.info().remote_address().to_string(),
            event.info().remote_port().to_string()
        );
        let hash_digest = md5::compute(hash_str);
        let player_uuid = Uuid::from_bytes(hash_digest.0);
        match (event.old_state(), event.info().state()) {
            // player tries to connect
            (
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_None,
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting,
            ) => {
                if let Some(cb) = &callbacks.on_connection_changed_callback{
                    cb(&player_uuid, ConnectionState::Connecting);
                }
                let should_accept = (callbacks.on_connect_requested_callback)(&player_uuid);
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
                if active_connetions.contains_key(&player_uuid){
                    active_connetions.remove(&player_uuid);
                }
                if let Some(cb) = &callbacks.on_connection_changed_callback {
                    cb(&player_uuid, ConnectionState::Disconnected);
                }
            }
            // player connected
            (
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting,
                ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connected,
            ) => {
                if let Some(cb) = &callbacks.on_connection_changed_callback {
                    cb(&player_uuid, ConnectionState::Connected);
                }
            }

            (_, _) => (),
        }
        Ok(())
    }
    fn process_messages(&self, event: &GnsNetworkMessage<ToReceive>) -> ServerResult<()> {
        let data = event.payload();
        println!("{:?}", data);
        Ok(())
    }
    pub fn send(&self, player: Uuid, data: &Vec<u8>) -> ServerResult<()> {
        Ok(())
    }
    pub fn send_reliable(&self, player: Uuid, data: &Vec<u8>) -> ServerResult<()> {
        Ok(())
    }

    pub fn broadcast(&self, data: &Vec<u8>) -> ServerResult<()> {
        Ok(())
    }
    pub fn broadcast_reliable(&self, data: &Vec<u8>) -> ServerResult<()> {
        Ok(())
    }

    pub fn register_on_connect_requested(
        &mut self,
        callback: impl Fn(&Uuid) -> bool + 'static + Send,
    ) {
        self.callbacks.on_connect_requested_callback = Box::from(callback);
    }
    pub fn register_on_connection_state_changed(
        &mut self,
        callback: impl Fn(&Uuid, ConnectionState) + 'static + Send,
    ) {
        self.callbacks.on_connection_changed_callback = Some(Box::from(callback));
    }
    pub fn register_on_message(&mut self, callback: impl Fn(&Uuid, i32, Vec<u8>) + 'static + Send) {
        self.callbacks.on_message_callback = Some(Box::from(callback));
    }
}

impl<'a> Debug for Server<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("ip", &self.ip)
            .field("port", &self.port)
            .field("active_connetions", &self.active_connetions)
            .finish()
    }
}
