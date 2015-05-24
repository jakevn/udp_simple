use bitbuf::BitBuf;
use std::ptr;

pub struct UdpReliable {
    write_buff: BitBuf,
    ordered_buff: Vec<BitBuf>,
    unacked_sent: Vec<BitBuf>,
    newest_remote_seq: u16,
    local_seq: u16,
    ack_hist: u64,
    last_send_time: u32,
    last_recv_time: u32,
    last_remote_seq: u16,
    recv_since_last_send: u32,
}

struct ReliableHeader {
    obj_seq: u16,
    ack_seq: u16,
    ack_hist: u64,
    ack_time: u16,
    send_time: u32,
}

impl UdpReliable {

    pub fn new() -> UdpReliable {
        UdpReliable {
            write_buff: BitBuf::with_len(1400),
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
    pub fn get_buff_for_sending(&mut self, time: u32) -> Option<&BitBuf> {
        if self.write_buff.pos <= 19 {
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
            (time as u16 - self.last_recv_time as u16)
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

    pub fn reset_buff(&mut self) {
        self.advance_local_seq();
        self.recv_since_last_send = 0;
        let mut write_buff = BitBuf::with_len(1400);
        unsafe {
            let buff_ptr: *mut BitBuf = &mut write_buff;
            let self_buff_ptr: *mut BitBuf = &mut self.write_buff;
            ptr::swap(buff_ptr, self_buff_ptr);
        }
        self.unacked_sent.push(write_buff);
        self.buff_sent = false;
        self.buff_pos = 19;
    }
}

fn read_header(buf: &mut BitBuf) -> ReliableHeader {
    ReliableHeader {
        obj_seq: trim_seq(buf.read_u16()),
        ack_seq: trim_seq(buf.read_u16()),
        ack_hist: buf.read_u64(),
        ack_time: buf.read_u16(),
        send_time: 0,
    }
}

fn write_header(header: &ReliableHeader, buf: &mut BitBuf) {
    buf.write_u16(pad_seq(header.obj_seq));
    buf.write_u16(pad_seq(header.ack_seq));
    buf.write_u64(header.ack_hist);
    buf.write_u16(header.ack_time);
}

fn trim_seq(seq: u16) -> u16 {
   seq >> 1
}

fn pad_seq(seq: u16) -> u16 {
   let result = seq << 1;
   result | ((1 << 1) - 1)
}


