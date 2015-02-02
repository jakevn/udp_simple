
use std::old_io::net::udp::UdpSocket;
use std::old_io::net::ip::SocketAddr;
use std::collections::HashSet;
use std::collections::HashMap;
use std::time::duration::Duration;

use time::SteadyTime;

use udp_conn::UdpConn;
use udp_sock::UdpEvent::*;

pub struct UdpSock {
    udp: UdpSocket,
    conns: HashMap<SocketAddr, UdpConn>,
    reject_addrs: HashSet<SocketAddr>,
    conn_attempts: HashMap<SocketAddr, ConnectAttempt>,
    buf: [u8; 1400],
    config: UdpConfig,
    start_time: SteadyTime,
}

pub struct UdpConfig {
    // The max number of times to retry connect request before
    // connect attempt is considered a failure:
    max_connect_retry: u32,
    // Amount of time to wait before retrying connect request:
    connect_retry_delay: Duration,
    // Maximum number of inbound (we received connect request) connections.
    // Usually 0 for a client:
    max_inc_conns: u32,
    // If the last message received from a client exceeds this amount,
    //  the client will be dropped due to timeout:
    timeout: Duration,
}

pub struct ConnectAttempt {
    // The SocketAddr we are attempting to connect to:
    addr: SocketAddr,
    // The last time a connect request was sent:
    last_attempt: Duration,
    // The number of retries (initial attempt is 0, 1 added for each retry):
    retry_count: u32,
    // The entirety of data that is sent for the connect request:
    data: [u8; 1400],
    // The number of bytes to send.
    data_size: usize,
}

pub enum UdpEvent {
    Connected(SocketAddr),
    Disconnected(SocketAddr),
    ConnectRequestData(SocketAddr, [u8; 1400], u16),
    ConnectRequest(SocketAddr),
    Received(SocketAddr, [u8; 1400], u16),
    ConnectFail(SocketAddr, FailReason),
    ReceiveFail(SocketAddr, FailReason),
    SendFail(FailReason),
    Nil,
}

#[derive(Debug)]
pub enum FailReason {
    Overflow, // Packet size exceeds max MTU
    Malformed, // Packet empty or header invalid
    NotConnected, // Cannot complete action without active connection
    SocketErr, // Socket error at the OS level
    Timeout, // Action took longer than permitted
    Denied, // Action was denied by policy
}

// Command signals:
pub const CON_REQ: u8 = 2u8;
pub const DISC: u8 = 3u8;
pub const PING: u8 = 4u8;
pub const DENY: u8 = 5u8;
pub const ACCEPT: u8 = 6u8;
pub const UNRELIABLE: u8 = 7u8;
pub const RELIABLE: u8 = 8u8;

impl UdpSock {

    /// Poll the socket for incoming packets and return corresponding UdpEvent.
    /// UdpSocket should be polled frequently enough that the OS socket's incoming
    /// buffer does not become full, resulting in packet loss, and so that time
    /// sensitive applications do not suffer from delayed responses.
    pub fn poll(&mut self) -> UdpEvent {
        match self.udp.recv_from(&mut self.buf) {
            Ok((_, src)) if self.banned(&src) => {
                self.poll() // Address is on reject list, skip
            },
            Ok((amt, src)) if amt > 1 && self.connected(&src) => { // If greater than 1, not a command
                Received(src, self.buf, amt as u16)
            },
            Ok((amt, src)) if amt > 1 && self.buf[0] == CON_REQ => {
                ConnectRequestData(src, self.buf, amt as u16)
            },
            Ok((amt, src)) if amt == 1 => { // 1 byte received, process command
                match self.buf[0] {
                    CON_REQ => ConnectRequest(src),
                    DISC => Disconnected(src),
                    DENY => ConnectFail(src, FailReason::Denied),
                    ACCEPT if self.conn_attempts.contains_key(&src) => {
                        let time = self.elapsed_time();
                        self.conn_attempts.remove(&src);
                        self.conns.insert(src, UdpConn::new(src, time));
                        Connected(src)
                    }
                    _ => self.poll()
                }
            },
            Ok((_, src)) => {
                ReceiveFail(src, FailReason::Malformed)
            }
            _ => Nil, // No more incoming datagrams to process
        }
    }

    /// Performs periodic duties, including sending buffered outgoing data,
    /// checking for connection timeouts, checking status of in-progress connection
    /// attempts, and sending acks for received packets. Update should be called
    /// as often as appropriate considering the delay sensitivity for the application.
    pub fn update(&mut self) {
        self.update_conn_attempts();
        self.send_buffered_reliable();
        self.send_buffered_unreliable();
        self.check_for_timeouts();
    }

    fn update_conn_attempts(&mut self) {
        let mut retry: Vec<SocketAddr> = Vec::new();
        let mut cancel: Vec<SocketAddr> = Vec::new();
        for (adr, ca) in self.conn_attempts.iter() {
            if ca.last_attempt - self.elapsed_time() > self.config.connect_retry_delay {
                if ca.retry_count < self.config.max_connect_retry {
                    retry.push(*adr);
                } else {
                    cancel.push(*adr);
                }
            }
        }
        self.perform_retries(retry);
        self.cancel_conns(cancel);
    }

    fn perform_retries(&mut self, retry: Vec<SocketAddr>) {
        let time = self.elapsed_time();
        for a in retry.iter() {
            let c = self.conn_attempts.get_mut(a).unwrap();
            c.retry_count += 1;
            let data = c.data;
            let amt = c.data_size;
            self.udp.send_to(&data[0..amt], *a);
        }
    }

    fn cancel_conns(&mut self, cancel: Vec<SocketAddr>) {
        for c in cancel.iter() {
            self.conn_attempts.remove(c);
            // TODO: Add event for ConnectFail
        }
    }

    fn send_buffered_reliable(&mut self) {
        let time = self.elapsed_time();
        let mut send: Vec<(SocketAddr, &[u8])> = Vec::new();
        for (a, c) in self.conns.iter_mut() {
            match c.try_get_reliable_buff(time) {
                Some(b) => { send.push((*a, b)); },
                None => (),
            };
        }
        for &(a, b) in send.iter() {
            self.udp.send_to(b, a);
        }
    }

    fn send_buffered_unreliable(&mut self) {
        let time = self.elapsed_time();
        let mut send: Vec<(SocketAddr, &[u8])> = Vec::new();
        for (a, c) in self.conns.iter_mut() {
            match c.try_get_unreliable_buff(time) {
                Some(b) => { send.push((*a, b)); },
                None => (),
            }
        }
        for &(a, b) in send.iter() {
            self.udp.send_to(b, a);
        }
    }

    fn check_for_timeouts(&mut self) {
        let time = self.elapsed_time();
        let mut disc: Vec<SocketAddr> = Vec::new();
        for (a, c) in self.conns.iter_mut() {
            if c.timed_out(time) {
                disc.push(*a);
            }
        }
        for a in disc.iter() {
            self.disconnect(*a);
        }
    }

    /// The duration of time since the socket was created.
    #[inline]
    pub fn elapsed_time(&self) -> Duration {
        self.start_time - SteadyTime::now()
    }

    /// Returns true if the supplied SocketAddr is connected.
    pub fn connected(&self, adr: &SocketAddr) -> bool {
        self.conns.contains_key(adr)
    }

    /// Returns true if the supplied SocketAddr is banned.
    pub fn banned(&self, adr: &SocketAddr) -> bool {
        self.reject_addrs.contains(adr)
    }

    pub fn connect(&mut self, adr: SocketAddr) {
        if !self.conn_attempts.contains_key(&adr) {
            let mut data = [0u8; 1400];
            data[0] = CON_REQ;
            self.start_connect(adr, data, 1);
        }
    }

    pub fn connect_with_data(&mut self, adr: SocketAddr, data: &[u8]) {
        if data.len() > 1399 || self.conn_attempts.contains_key(&adr) {
            return;
        }
        let len = data.len();
        let mut buf = [0u8; 1400];
        buf[0] = CON_REQ;
        for i in 1..len {
            buf[i] = data[i - 1];
        }
        self.start_connect(adr, buf, len + 1);
    }

    fn start_connect(&mut self, adr: SocketAddr, data: [u8; 1400], amt: usize) {
        self.udp.send_to(&data[0..amt], adr);
        let curr_time = self.elapsed_time();
        self.conn_attempts.insert(adr, ConnectAttempt {
            addr: adr,
            last_attempt: curr_time,
            retry_count: 0,
            data: data,
            data_size: amt,
        });
    }

    /// If currently trying to connect to provided SocketAddr,
    /// connection process will be cancelled and any connection
    /// reply will be ignored.
    pub fn cancel_connect(&mut self, adr: SocketAddr) {
        if self.conn_attempts.contains_key(&adr) {
            self.conn_attempts.remove(&adr);
            // TODO: Gracefully handle potential future acceptance command from target
            // so that target does not need to attempt to keepalive already dead conn.
        }
    }

    /// If connected to provided SocketAddr, sends disconnect command and
    /// removes from connection list.
    pub fn disconnect(&mut self, adr: SocketAddr) {
        if self.conns.contains_key(&adr) {
            self.udp.send_to(&[DISC], adr);
            self.conns.remove(&adr);
            // TODO: Add graceful disconnect (use reliable and await response) so
            // that disconnect message is not lost and immediate future reconnects
            // are not hindered by potential bad remote state.
        }
    }

    /// Accepts the incoming connection request for the supplied SocketAddr.
    pub fn accept(&mut self, adr: SocketAddr) {
        self.udp.send_to(&[ACCEPT], adr);
    }

    /// Rejects the incoming connection request for the supplied SocketAddr.
    pub fn reject(&mut self, adr: SocketAddr) {
        self.udp.send_to(&[DENY], adr);
    }

    /// Ban the provided SocketAddr, ignoring all future incoming transmissions from it.
    pub fn ban(&mut self, adr: SocketAddr) {
        self.reject_addrs.insert(adr);
    }

    /// Remove the supplied SocketAddr from the ban list.
    pub fn unban(&mut self, adr: SocketAddr) {
        self.reject_addrs.remove(&adr);
    }

    /// Send a datagram directly, bypassing all layers of udp_simple.
    pub fn send_raw(&mut self, adr: SocketAddr, buf: &[u8]) {
        self.udp.send_to(&*buf, adr);
    }

    /// Queue an unreliable message for sending. If not connected to SocketAddr, ignored.
    /// If the current outbound buffer for the associated connection is full, it will be 
    /// automatically sent and a new outbound buffer will be prepared and used.
    pub fn queue_unreliable(&mut self, adr: SocketAddr, buf: &[u8]) {
        let duration = self.elapsed_time();
        match self.conns.get_mut(&adr) {
            Some(c) => {
                match c.queue_or_send_unreliable(buf, duration) {
                    Some(b) => {
                        self.udp.send_to(b, adr);
                    },
                    None => (),
                }
            },
            None => (),
        }
    }

    /// Queue a reliable message for sending. If not connected to SocketAddr, ignored.
    /// If the current outbound buffer for the associated connection is full, it will be 
    /// automatically sent and a new outbound buffer will be prepared and used.
    pub fn queue_reliable(&mut self, adr: SocketAddr, buf: &[u8]) {
        let duration = self.elapsed_time();
        match self.conns.get_mut(&adr) {
            Some(c) => {
                match c.queue_or_send_reliable(buf, duration) {
                    Some(b) => {
                        self.udp.send_to(b, adr);
                    },
                    None => (),
                }
            },
            None => (),
        }
    }

}