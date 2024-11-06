
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

pub mod messages{
    include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));
}
