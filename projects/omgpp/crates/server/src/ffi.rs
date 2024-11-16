use crate::Server;
use std::{ffi::{c_char, CStr}, net::IpAddr, str::FromStr, ptr::null_mut};

#[no_mangle]
pub unsafe extern "C"  fn server_create(ip:*const c_char, port:u16) -> *mut Server<'static> {
    let c_string = CStr::from_ptr(ip).to_str();
    if c_string.is_err() {
        return null_mut();
    }

    if let Some(addres) = IpAddr::from_str(c_string.unwrap()).ok(){
        let server_res = Server::new(addres, port);
        match server_res {
            Ok(server) => Box::into_raw(Box::from(server)),
            Err(_) => null_mut()
        }
    }else {
        null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn server_process(server:*mut Server){
    _ = server.as_mut().unwrap().process::<128>();
}
