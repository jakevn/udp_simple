use std::io::net::ip::SocketAddr;

pub struct UdpConn {
	ep: SocketAddr,
	uc: UnreliableChannel,
	rc: ReliableChannel,
}

struct ReliableHeader {
	obj_seq: u16,
	ack_seq: u16,
	ack_hist: u64,
	ack_time: u16,
	send_time: u32,
}

struct UnreliableChannel {
	write_buff: [u8, ..1400],
}

struct ReliableChannel {
	write_buff: [u8, ..1400],
	ordered_buff: Vec<[u8, ..1400]>,
	unacked_sent: Vec<[u8, ..1400]>,
	newest_remote_seq: u16,
	local_seq: u16,
	ack_hist: u64,
	last_send_time: u32,
	last_recv_time: u32,
	last_remote_seq: u16,
	recv_since_last_send: u32,
}