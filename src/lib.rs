#![feature(slicing_syntax)]

#![crate_name = "udp_simple"]
#![crate_type="lib"]

pub use socket::*;
pub use connection::*;
pub use udp_sock::*;
pub use udp_conn::*;

pub mod socket;
pub mod connection;
pub mod udp_sock;
pub mod udp_conn;