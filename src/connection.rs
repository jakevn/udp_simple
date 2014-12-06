use std::io::net::ip::SocketAddr;

#[deriving(Show)]
pub struct Connection {
    pub addr: SocketAddr,
    pub id: u64,
}

impl Connection {
    pub fn new(addr: SocketAddr, id: u64) -> Connection {
        Connection {
            addr: addr,
            id: id,
        }
    }
}