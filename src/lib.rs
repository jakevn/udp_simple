#![feature(slicing_syntax)]

#![crate_name = "udp_simple"]
#![crate_type="lib"]

extern crate bitbuf;
extern crate time;

pub use udp_sock::*;

pub mod udp_sock;

use udp_conn::*;
use udp_reliable::*;

mod udp_conn;
mod udp_reliable;