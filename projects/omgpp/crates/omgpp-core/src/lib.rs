
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

pub mod ffi;

use std::{net::IpAddr, sync::LazyLock};

use either::Either;
use gns::{GnsGlobal, GnsUtils, GnsDroppable, IsReady, GnsConnection, GnsSocket, GnsConnectionInfo};

pub mod messages{
    include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));
}

#[allow(dead_code)]
#[derive(Debug,Clone,Hash,PartialEq,Eq)]
#[repr(i16)]
pub enum ConnectionState {
    None = -1,
    Disconnected = 0,
    Disconnecting = 1,
    Connecting = 2,
    Connected = 3,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Endpoint {
    pub ip: IpAddr,
    pub port: u16,
}

pub struct GnsWrapper {
    pub global: GnsGlobal,
    pub utils: GnsUtils,
}
unsafe impl Send for GnsWrapper {}
unsafe impl Sync for GnsWrapper {}

pub static GNS: LazyLock<Result<GnsWrapper,String>> = LazyLock::new(|| {
    Ok(GnsWrapper {
        global: GnsGlobal::get()?,
        utils: GnsUtils::new().ok_or("Error occurred when creating GnsUtils")?,
    })
});

pub trait ToEndpoint {
    fn to_endpoint(&self) -> Endpoint;
}
impl ToEndpoint for GnsConnectionInfo {
    fn to_endpoint(&self) -> Endpoint {
        Endpoint {
            ip: IpAddr::V6(self.remote_address()),
            port: self.remote_port(),
        }
    }
}


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