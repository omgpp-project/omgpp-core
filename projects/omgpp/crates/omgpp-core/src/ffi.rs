use std::net::IpAddr;
use super::Endpoint;
use uuid::Uuid;

pub trait ToFfi<T> {
    fn to_ffi(&self) -> T;
}
impl ToFfi<EndpointFFI> for Endpoint {
    fn to_ffi(&self) -> EndpointFFI {
        EndpointFFI { 
            ipv6_octets: match self.ip {
                IpAddr::V4(ipv4_addr) => ipv4_addr.to_ipv6_mapped().octets(),
                IpAddr::V6(ipv6_addr) => ipv6_addr.octets(),
            },
            port:self.port,
        }
    }
}
impl ToFfi<UuidFFI> for Uuid {
    fn to_ffi(&self) -> UuidFFI {
        UuidFFI { 
            bytes:self.into_bytes(),
        }
    }
}

#[repr(C, packed)]
pub struct EndpointFFI {
    ipv6_octets: [u8;16],
    port:u16
}

#[repr(C,packed)]
pub struct UuidFFI {
    bytes:[u8;16]
}