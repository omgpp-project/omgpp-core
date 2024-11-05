use std::{
    fmt::{Debug, Error},
    net::{IpAddr, Ipv6Addr},
    ops::Deref,
    rc::Rc,
    str::FromStr,
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::{self, JoinHandle},
};

use gns::{GnsGlobal, GnsSocket, GnsUtils, IsCreated, IsServer};
use uuid::Uuid;

type OnConnectRequestCallback = Box<dyn FnMut(&Uuid) -> bool + Send + 'static>;
type OnConnectionChangedCallback = Box<dyn FnMut(&Uuid, ConnectionState) + Send + 'static>;
type OnMessageCallback = Box<dyn FnMut(&Uuid, i32, Vec<u8>) + Send + 'static>;

type ServerResult<T> = Result<T, String>; // TODO replace error with enum

static GNS_INIT: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
pub enum ConnectionState {
    Disconnected = 0,
    Disconnecting = 1,
    Connecting = 2,
    Connected = 3,
}

pub struct Server {
    ip: IpAddr,
    port: u16,
    active_connetions: Vec<Uuid>,
    thread: Option<JoinHandle<()>>,
    should_terminate_thread: Arc<AtomicBool>,
    on_connect_requested_callback: Option<OnConnectRequestCallback>,
    on_connection_changed_callback: Option<OnConnectionChangedCallback>,
    on_message_callback: Option<OnMessageCallback>,
}

struct ServerThreadHandler<'a> {
    value: &'a Server,
}
impl<'a> Deref for ServerThreadHandler<'a> {
    type Target = Server;

    fn deref(&self) -> &Self::Target {
        return self.value;
    }
}
unsafe impl<'a> Send for ServerThreadHandler<'a> {}
unsafe impl<'a> Sync for ServerThreadHandler<'a> {}

 
impl Server {
    pub fn new(ip: IpAddr, port: u16) -> Server {
        Server {
            ip,
            port,
            thread: None,
            should_terminate_thread: Arc::new(AtomicBool::new(false)),
            active_connetions: vec![],
            on_connect_requested_callback: None,
            on_connection_changed_callback: None,
            on_message_callback: None,
        }
    }

    pub fn start(&'static mut self) -> ServerResult<()> {
        if let Some(_) = &self.thread {
            return Err("Cannot start a server. Server already running".to_string());
        }
        let gns_global = GnsGlobal::get()?;
        let gns_utils = GnsUtils::new().ok_or("Error occurred when creating GnsUtils")?;

        let gns_socket = GnsSocket::<IsCreated>::new(&gns_global, &gns_utils).unwrap();
        let mut address_to_bind = match self.ip {
            IpAddr::V4(v4) => v4.to_ipv6_mapped(),
            IpAddr::V6(v6) => v6,
        };
        let server_socket = gns_socket
            .listen(address_to_bind, self.port)
            .or(ServerResult::Err("Create server socket".to_string()))?;
        self.thread = Some(thread::spawn(Server::server_thread_executor(self.should_terminate_thread, )));
        Ok(())
    }
    pub fn stop(&self) -> ServerResult<()> {
        self.should_terminate_thread.store(false, Ordering::Relaxed);
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
        callback: impl FnMut(&Uuid) -> bool + 'static + Send,
    ) {
        self.on_connect_requested_callback = Some(Box::from(callback));
    }
    pub fn register_on_connection_state_changed(
        &mut self,
        callback: impl FnMut(&Uuid, ConnectionState) + 'static + Send,
    ) {
        self.on_connection_changed_callback = Some(Box::from(callback));
    }
    pub fn register_on_message(
        &mut self,
        callback: impl FnMut(&Uuid, i32, Vec<u8>) + 'static + Send,
    ) {
        self.on_message_callback = Some(Box::from(callback));
    }

    fn server_thread_executor(cancellation_token: Arc<AtomicBool>, socket: Arc<GnsSocket<IsServer>>,) {
        while cancellation_token.load(Ordering::Relaxed) != true {

        }
    }
}

impl Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("ip", &self.ip)
            .field("port", &self.port)
            .field("active_connetions", &self.active_connetions)
            .finish()
    }
}
