#![feature(globs)]
#![feature(slicing_syntax)]

#![crate_name = "udp_simple"]
#![crate_type="lib"]

pub use socket::*;
pub use connection::*;
pub use internal_socket::*;
pub use internal_connection::*;

pub mod socket;
pub mod connection;
pub mod internal_socket;
pub mod internal_connection;