use std::old_io::net::ip::SocketAddr;
use std::slice::bytes::copy_memory;
use udp_sock::UdpSock;
use udp_reliable::UdpReliable;
use std::time::duration::Duration;

pub struct UdpConn {
    address: SocketAddr,
    unreliable: UdpUnreliable,
    reliable: UdpReliable,
    last_recv_time: Duration,
}

struct UdpUnreliable {
    write_buff: [u8; 1400],
}

impl UdpConn {

    pub fn new(adr: SocketAddr, curr_time: Duration) -> UdpConn {
        UdpConn {
            address: adr,
            unreliable: UdpUnreliable { write_buff: [0u8; 1400] },
            reliable: UdpReliable::new(),
            last_recv_time: curr_time,
        }
    }

    pub fn queue_or_send_reliable(&mut self, data: &[u8], curr_time: Duration) -> Option<&[u8]> {
        None
    }

    pub fn queue_or_send_unreliable(&mut self, data: &[u8], curr_time: Duration) -> Option<&[u8]> {
        None
    }

    pub fn try_receive_reliable(&mut self, data: &[u8], curr_time: Duration) -> Option<&[u8]> {
        None
    }

    pub fn try_receive_unreliable(&mut self, data: &[u8], curr_time: Duration) -> Option<&[u8]> {
        None
    }

    pub fn reliable_data_queued(&self) -> bool {
        false
    }

    pub fn try_get_reliable_buff(&mut self, curr_time: Duration) -> Option<&[u8]> {
        None
    }

    pub fn try_get_unreliable_buff(&mut self, curr_time: Duration) -> Option<&[u8]> {
        None
    }

    pub fn unreliable_data_queued(&self) -> bool {
        false
    }

    pub fn timed_out(&self, curr_time: Duration) -> bool {
        false
    }
}
