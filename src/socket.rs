use std::io::net::udp::UdpSocket;
use std::io::net::ip::SocketAddr;
use std::str::FromStr;

use connection::Connection;
use udp_sock::UdpSock;

// Socket contains the API implementation. It provides
// an interface for sending/receiving from/to and configuring
// the internal socket.
pub struct Socket {
    pub address: SocketAddr,
    com_send: Sender<SocketCommand>,
    event_recv: Receiver<SocketEvent>,
    packet_send: Sender<Packet>,
    packet_recv: Receiver<Packet>,
    connections: Vec<Connection>,
}

pub struct SocketConfig {
    max_conn: u16, // The maximum number of incoming connections
    timeout_ms: u16, // In milliseconds, how long until disconnect due to timeout
    heartbeat_ms: u16, // If haven't sent to a connection in this many milliseconds, send heartbeat
    mtu: u16, // Maximum packet size
}

pub struct Packet {
    pub connection: Connection,
    pub size: uint,
    pub data: [u8, ..1400],
}

pub const DEFAULT_SERVER: SocketConfig = SocketConfig {
    max_conn: 1024,
    timeout_ms: 5000,
    heartbeat_ms: 1000,
    mtu: 1400,
};

pub const DEFAULT_CLIENT: SocketConfig = SocketConfig {
    max_conn: 0,
    timeout_ms: 5000,
    heartbeat_ms: 1000,
    mtu: 1400,
};

pub enum SocketEvent {
    Connected(Connection),
    Disconnected(Connection),
    ConnectRequest(SocketAddr),
    Received(Packet),
    ConnectFail(SocketAddr, FailReason),
    ReceiveFail(Connection, FailReason),
    SendFail(FailReason),
}

pub enum SocketCommand {
    Connect(SocketAddr),
    Disconnect(SocketAddr),
    AcceptConn(SocketAddr),
}

#[deriving(Show)]
pub enum FailReason {
    Overflow, // Packet size exceeds max MTU
    Malformed, // Packet empty or header invalid
    NotConnected, // Cannot complete action without active connection
    SocketErr, // Socket error at the OS level
    Timeout, // Action took longer than permitted
    Denied, // Action was denied by policy
}

impl Iterator<SocketEvent> for Socket {
    fn next(&mut self) -> Option<SocketEvent> {
        self.poll()
    }
}

impl Socket {

    pub fn create(address: &str) -> Option<Socket> {
        let addr: Option<SocketAddr> = FromStr::from_str(address);
        match addr {
            Some(a) => match UdpSocket::bind(a) {
                Ok(mut s) => {
                    Some(Socket::new_udp(s, a))
                }
                Err(e) => None
            },
            None => None
        }
    }

    fn new_udp(mut s: UdpSocket, a: SocketAddr) -> Socket {
        s.set_read_timeout(Some(0));
        let (pack_send_tx, pack_send_rx) = channel();
        let (pack_recv_tx, pack_recv_rx) = channel();
        let (comm_tx, comm_rx) = channel();
        let (event_tx, event_rx) = channel();
        let internal_sock = UdpSock {
            udp: s,
            udp_conns: Vec::new(),
            com_recv: comm_rx,
            event_send: event_tx,
            packet_send_rx: pack_send_rx,
            packet_recv_tx: pack_recv_tx,
        };
        spawn(proc() { internal_sock.poll_loop() } );
        Socket {
            address: a,
            com_send: comm_tx,
            event_recv: event_rx,
            packet_send: pack_send_tx,
            packet_recv: pack_recv_rx,
            connections: Vec::new(),
        }
    }

    pub fn accept_connection(&mut self, addr: SocketAddr) {
        self.com_send.send(SocketCommand::AcceptConn(addr));
    }

    pub fn connect(&mut self, to_addr: &str) {
        self.com_send.send(SocketCommand::Connect(FromStr::from_str(to_addr).unwrap()));
    }

    pub fn send_unreliable(&mut self, data: [u8, ..1400], to: &Connection) {
        self.send(data, to.addr);
    }

    pub fn send_reliable(&mut self, data: [u8, ..1400], to: Connection) {
        self.send(data, to.addr);
    }

    fn send(&mut self, data: [u8, ..1400], addr: SocketAddr) {
        self.packet_send.send(Packet {
            connection: Connection::new(addr, 0),
            size: 1400, data: data
        });
    }

    pub fn poll(&mut self) -> Option<SocketEvent> {
        match self.packet_recv.try_recv() {
            Ok(pkt) => return Some(SocketEvent::Received(pkt)),
            _ => {},
        };
        match self.event_recv.try_recv() {
            Ok(evnt) => return Some(evnt),
            _ => {},
        };
        None
    }
}