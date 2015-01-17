extern crate udp_simple;

use std::io::timer;
use std::time::Duration;

use udp_simple::{Socket,Connection,SocketEvent};

struct SockTest {
    received: u64,
    conn: Option<Connection>,
}

fn main() {
    /*match Socket::create("127.0.0.1:9001") {
        Some(s) => { socket_loop(s); },
        None => {
            // println!("Failed to create socket.") 
        },
    };*/
}
/*
fn socket_loop(mut sock: Socket) {
    sock.connect("127.0.0.1:9000");
    let mut socktest = SockTest {received: 0, conn: None};
    loop {
        'poll: loop {
            match sock.poll() {
                Some(e) => { socktest = route_socket_event(e, socktest, &mut sock); },
                None => break 'poll,
            };
        }
        match socktest.conn {
            Some(c) => send_ten_times(c, &mut sock),
            None => {},
        };
        
        timer::sleep(Duration::milliseconds(15));
    }
}

fn send_ten_times(c: Connection, sock: &mut Socket) {
    for _ in range(0i, 1) {
        sock.send_reliable([0u8; 1400], c);
    }
}

fn route_socket_event(ev: SocketEvent, mut st: SockTest, sock: &mut Socket) -> SockTest {
    match ev {
        SocketEvent::ConnectRequest(ep) => {
            //println!("Auto-approving connect request from {}", ep);
            sock.accept_connection(ep);
        },
        SocketEvent::Connected(c) => {
            //println!("Connected to: {}", c);
            st.conn = Some(c);
        },
        SocketEvent::Disconnected(c) => {
            //println!("Disconnected from: {}", c);
        },
        SocketEvent::Received(p) => {
            st.received = st.received + 1;
            //println!("Received packet from: {}", p.connection);
        },
        SocketEvent::ConnectFail(c, e) => {
            //println!("Failed to connect to: {}, {}", c, e);
        },
        SocketEvent::ReceiveFail(c, e) => {
            //println!("Failed to receive from: {}, {}", c, e);
        },
        SocketEvent::SendFail(e) => {
            //println!("Failed to send to: {}", e);
        },
    };
    st
}*/