use omgpp_core::{
    ffi::{EndpointFFI, ToFfi, UuidFFI},
    ConnectionState,
};
use std::{
    ffi::{c_char, c_uchar, CStr},
    net::IpAddr,
    ptr::null_mut,
    str::FromStr,
};
use uuid::Uuid;
use crate::server::Server;


// FFI
type ServerOnConnectRequested = extern "C" fn(UuidFFI, EndpointFFI) -> bool;
type ServerOnConnectionChanged = extern "C" fn(UuidFFI, EndpointFFI, ConnectionState);
type ServerOnMessage = extern "C" fn(UuidFFI, EndpointFFI, i64, *const c_uchar, usize);
type ServerOnRpc = extern "C" fn(UuidFFI, EndpointFFI,bool, i64, u64, i64, *const c_uchar,usize);

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
        .register_on_connect_requested(move |_server,uuid, endpoint| {
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
        .register_on_connection_state_changed(move |_server, uuid, endpoint, state| {
            callback(uuid.to_ffi(), endpoint.to_ffi(), state)
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
        .register_on_message(move |_server,uuid, endpoint, message_id, data| {
            callback(
                uuid.to_ffi(),
                endpoint.to_ffi(),
                message_id,
                data.as_ptr(),
                data.len(),
            )
        });
}
#[no_mangle]
pub unsafe extern "C" fn server_register_on_rpc(
    server: *mut Server,
    callback: ServerOnRpc,
) {
    server
        .as_mut()
        .unwrap()
        .register_on_rpc(move |_server,uuid, endpoint, reliable, method_id, request_id, arg_type, arg_data| {
            callback(
                uuid.to_ffi(),
                endpoint.to_ffi(),
                reliable,
                method_id,
                request_id,
                arg_type,
                arg_data.as_ptr(),
                arg_data.len(),
            )
        });
}
#[no_mangle]
pub unsafe extern "C" fn server_send(
    server: *mut Server,
    uuid: *const UuidFFI,
    msg_type: i64,
    data: *const c_uchar,
    offset: isize,
    size: usize,
) {

    let msg_data = core::slice::from_raw_parts(data.offset(offset), size);
    let client_uuid = uuid_from_ffi_ptr(uuid);
    _ = server
        .as_ref()
        .unwrap()
        .send(&client_uuid, msg_type, msg_data)
}

#[no_mangle]
pub unsafe extern "C" fn server_send_reliable(
    server: *mut Server,
    uuid: *const UuidFFI,
    msg_type: i64,
    data: *const c_uchar,
    offset: isize,
    size: usize,
) {
    let msg_data = core::slice::from_raw_parts(data.offset(offset), size);
    let client_uuid = uuid_from_ffi_ptr(uuid);
    _ = server
        .as_ref()
        .unwrap()
        .send_reliable(&client_uuid, msg_type, msg_data)
}
#[no_mangle]
pub unsafe extern "C" fn server_broadcast(
    server: *mut Server,
    msg_type: i64,
    data: *const c_uchar,
    offset: isize,
    size: usize,
) {
    let msg_data = core::slice::from_raw_parts(data.offset(offset), size);
    _ = server.as_ref().unwrap().broadcast(msg_type, msg_data)
}
#[no_mangle]
pub unsafe extern "C" fn server_broadcast_reliable(
    server: *mut Server,
    msg_type: i64,
    data: *const c_uchar,
    offset: isize,
    size: usize,
) {
    let msg_data = core::slice::from_raw_parts(data.offset(offset), size);
    _ = server
        .as_ref()
        .unwrap()
        .broadcast_reliable(msg_type, msg_data)
}
#[no_mangle]
pub unsafe extern "C" fn server_call_rpc(
    server: *mut Server,
    client: *const UuidFFI,
    reliable: bool,
    method_id: i64,
    request_id: u64,
    arg_type: i64,
    arg_data: *const c_uchar,
    arg_data_offset: isize,
    arg_data_size: usize,
) {
    let client_uuid = uuid_from_ffi_ptr(client);
    let msg_data = match arg_data_size {
        0 => None,
        _ => Some(core::slice::from_raw_parts(arg_data.offset(arg_data_offset), arg_data_size)),
    };
    _ = server.as_ref().unwrap().call_rpc(
        &client_uuid,
        reliable,
        method_id,
        request_id,
        arg_type,
        msg_data,
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_call_rpc_broadcast(
    server: *mut Server,
    reliable: bool,
    method_id: i64,
    request_id: u64,
    arg_type: i64,
    arg_data: *const c_uchar,
    arg_data_offset: isize,
    arg_data_size: usize,
) {
    let msg_data = match arg_data_size {
        0 => None,
        _ => Some(core::slice::from_raw_parts(arg_data.offset(arg_data_offset), arg_data_size)),
    };
    _ = server.as_ref().unwrap().call_rpc_broadcast(
        reliable,
        method_id,
        request_id,
        arg_type,
        msg_data,
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_disconnect(_server: *mut Server, uuid: *const UuidFFI) {
    let _client_uuid = uuid_from_ffi_ptr(uuid);

    panic!("server disconnect not implemented")
    // TODO uncomment when disconnect implemented
    // server.as_ref().unwrap().disconnect();
}
#[no_mangle]
#[allow(unreachable_patterns)]
pub unsafe extern "C" fn server_destroy(server: *mut Server) {
    match server.as_mut() {
        server_ref => {
            drop(server_ref);
        }
        _ => (),
    }
}

unsafe fn uuid_from_ffi_ptr(uuid_ffi: *const UuidFFI) -> Uuid {
    Uuid::from_bytes(uuid_ffi.as_ref().unwrap().bytes)
}
