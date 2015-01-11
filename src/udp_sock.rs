use std::io::timer;
use std::time::Duration;
use std::io::net::udp::UdpSocket;
use std::io::net::ip::SocketAddr;
use std::sync::mpsc::*;

use socket::{Packet, FailReason, SocketCommand, SocketEvent};
use udp_conn::UdpConn;
use connection::Connection;
use socket::SocketEvent::*;
use socket::SocketCommand::*;

pub struct UdpSock {
    pub udp: UdpSocket,
    pub udp_conns: Vec<UdpConn>,
    pub com_recv: Receiver<SocketCommand>,
    pub event_send: Sender<SocketEvent>,
    pub packet_send_rx: Receiver<Packet>,
    pub packet_recv_tx: Sender<Packet>,
}

// Packet identifiers (first byte):
pub const CON_REQ: u8 = 2u8;
pub const DISC: u8 = 3u8;
pub const PING: u8 = 4u8;
pub const DENY: u8 = 5u8;
pub const ACCEPT: u8 = 6u8;

impl UdpSock {

	pub fn poll_loop(mut self) {

        let mut buf = [0u8; 1400]; // Received data buffer
        let mut reset_sleep_curr = false; // If sleep_curr should be reset to sleep_base
        let mut sleep_curr = 5; // Sleep time (ms) that increments by 1 for each uneventful loop
        let sleep_base = 0; // If we've received an event, reset sleep time to this
        let sleep_max = 20; // The max amount of sleep time to increment to

        loop {

            'command: loop {
                match self.com_recv.try_recv() {
                    Ok(c) => {
                        self.execute_command(c);
                    },
                    _ => break 'command,
                };
                reset_sleep_curr = true;
            }

            'receive: loop {
                match self.udp.recv_from(&mut buf) {
                    Ok((amt, src)) if amt > 1 => {
                        self.packet_recv_tx.send(Packet {
                        	connection: Connection::new(src, 0),
                        	size: amt as u16,
                        	data: buf
                        });
                    },
                    Ok((amt, src)) if amt > 0 => {
                    	self.event_send.send(UdpSock::parse_signal(src, buf[0]));
                    },
                    _ => break 'receive,
                };
                reset_sleep_curr = true;
            }

            'send: loop {
                match self.packet_send_rx.try_recv() {
                    Ok(pkt) => {
                        self.udp.send_to(&pkt.data, pkt.connection.addr);
                    },
                    _ => break 'send,
                };
                reset_sleep_curr = true;
            }

            sleep_curr =  if reset_sleep_curr { sleep_base } 
                          else if sleep_curr >= sleep_max { sleep_max }
                          else { sleep_curr + 1 };

            timer::sleep(Duration::milliseconds(sleep_curr));
            reset_sleep_curr = false;
        };
    }

    fn execute_command(&mut self, c: SocketCommand) {
        match c {
            Connect(adr) => {
            	self.udp.send_to(&[2u8], adr);
            },
            Disconnect(adr) => {
            	self.udp.send_to(&[3u8], adr);
            },
            AcceptConn(adr) => {
                self.udp.send_to(&[6u8],adr);
                self.event_send.send(Connected(Connection::new(adr, 0)));
            },
        };
    }

    fn parse_signal(src: SocketAddr, sig_id: u8) -> SocketEvent {
        match sig_id {
            CON_REQ => ConnectRequest(src),
            DISC => Disconnected(Connection::new(src, 0)),
            DENY => ConnectFail(src, FailReason::Denied),
            ACCEPT => Connected(Connection::new(src, 0)),
            _ => ReceiveFail(Connection::new(src, 0), FailReason::Malformed)
        }
    }

}