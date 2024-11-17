
use omgpp_core::{ffi::{EndpointFFI, ToFfi, UuidFFI}, ConnectionState};
use crate::Server;
use std::{
    ffi::{c_char, c_uchar, CStr},
    net::IpAddr,
    ptr::null_mut,
    str::FromStr,
};

// FFI
type ServerOnConnectRequested = extern "C" fn(UuidFFI, EndpointFFI) -> bool;
type ServerOnConnectionChanged = extern "C" fn(UuidFFI, EndpointFFI, ConnectionState);
type ServerOnMessage = extern "C" fn(UuidFFI, EndpointFFI, i64, *const c_uchar,usize);

#[no_mangle]
pub unsafe extern "C" fn server_create(ip: *const c_char, port: u16) -> *mut Server<'static> {
    let c_string = CStr::from_ptr(ip).to_str();
    if c_string.is_err() {
        return null_mut();
    }
    
    if let Some(addres) = IpAddr::from_str(c_string.unwrap()).ok() {
        let server_res = Server::new(addres, port);
        match server_res {
            Ok(server) => Box::into_raw(Box::from(server)),
            Err(_) => null_mut(),
        }
    } else {
        null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn server_process(server: *mut Server) {
    _ = server.as_mut().unwrap().process::<128>();
}
#[no_mangle]
pub unsafe extern "C" fn server_register_on_connect_requested(
    server: *mut Server,
    callback: ServerOnConnectRequested,
) {
    server
        .as_mut()
        .unwrap()
        .register_on_connect_requested(move |uuid, endpoint| {
            callback(uuid.to_ffi(), endpoint.to_ffi())
        });
}

#[no_mangle]
pub unsafe extern "C" fn server_register_on_connection_state_change(
    server: *mut Server,
    callback: ServerOnConnectionChanged,
) {
    server
        .as_mut()
        .unwrap()
        .register_on_connection_state_changed(move |uuid, endpoint,state| {
            callback(uuid.to_ffi(), endpoint.to_ffi(),state)
        });
}

#[no_mangle]
pub unsafe extern "C" fn server_register_on_message(
    server: *mut Server,
    callback: ServerOnMessage,
) {
    server
        .as_mut()
        .unwrap()
        .register_on_message(move |uuid, endpoint,message_id,data| {
            callback(uuid.to_ffi(), endpoint.to_ffi(),message_id,data.as_ptr(),data.len())
        });
}

#[no_mangle]
pub unsafe  extern "C" fn server_destroy(server: *mut Server)
{
    match server.as_mut() {
        server_ref => {
            drop(server_ref);
        }
        _ => (),
    }
}