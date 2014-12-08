use std::io::net::ip::SocketAddr;
use std::slice::bytes::copy_memory;
use std::num::FromPrimitive;

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
	buff_sent: bool,
	buff_pos: uint,
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

impl ReliableChannel {

	pub fn new() -> ReliableChannel {
		ReliableChannel {
			write_buff: [0, ..1400],
			buff_sent: false,
			buff_pos: 4,
			ordered_buff: Vec::new(),
			unacked_sent: Vec::new(),
			newest_remote_seq: 0,
			local_seq: 0,
			ack_hist: 0,
			last_send_time: 0,
			last_recv_time: 0,
			last_remote_seq: 0,
			recv_since_last_send: 0,
		}
	}

	// If we have a non-empty buffer, we will write the reliable header
	// and return the ready-to-send byte buffer and update state to reflect
	// that it is now sent. If buffer not ready to send, return None:
	pub fn get_buff_for_sending(&mut self, time: u32) -> Option<&[u8, ..1400]> {
		if self.buff_pos == 19 {
			None
		} else {
			self.buff_sent = true;
			let header = self.create_header(time);

			Some(&self.write_buff)
		}
	}

	// If too much time has elapsed since last sent a message, yet we have received
	// a message without acking it, we need to force an ack (that is, send a packet
	// that contains only a reliable header). If we do not need to send an ack, this
	// will simply return None:
	pub fn get_ack(&mut self, current_time: u32) -> Option<ReliableHeader> {
		if self.recv_since_last_send > 16 || 
			(self.recv_since_last_send > 0 && current_time - self.last_send_time > 33) {
			None
		} else {
			None
		}
	}

	fn create_header(&self, time: u32) -> ReliableHeader {
		let ackt: u16 = if self.last_recv_time > time {
			FromPrimitive::from_u32(time - self.last_recv_time).unwrap()
		} else { 
			0u16
		};
		ReliableHeader {
			obj_seq: self.local_seq,
			ack_seq: self.newest_remote_seq,
			ack_hist: self.ack_hist,
			ack_time: ackt,
			send_time: time,
		}
	}

	fn advance_local_seq(&mut self) {
		self.local_seq = (self.local_seq + 1) & 32767;
	}

	pub fn buff_has_room(&self, byte_count: uint) -> bool {
		self.buff_pos + byte_count <= 1400
	}

	pub fn add_to_buff(&mut self, bytes: [u8, ..1400]) {
		let mut p = 0;
		for i in range(self.buff_pos, bytes.len()) {
			self.write_buff[i] = bytes[p];
			p = p + 1;
			self.buff_pos = self.buff_pos + 1;
		}
	}

	fn read_header(buff: &[u8, ..1400]) -> ReliableHeader {
		
	}

	pub fn reset_buff(&mut self) {
		let sent_buff = self.write_buff;
		self.advance_local_seq();
		self.recv_since_last_send = 0;
		self.unacked_sent.push(sent_buff);
		self.write_buff = [0, ..1400];
		self.buff_sent = false;
		self.buff_pos = 19;
	}

}