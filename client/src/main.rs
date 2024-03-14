use std::net::UdpSocket;
use tftp_libs::{build_message, Message};

const SERVER_HOST: &str = "127.0.0.1:69";

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:34252").expect("Failed to bind to udp socket");

    let mut buf = [0; 512];
    let message = Message::ReadRequest {
        file_name: "test.txt".to_string(),
        mode: "Hello".to_string(),
    };
    let buffer = build_message(message);
    socket
        .send_to(&buffer, SERVER_HOST)
        .expect("couldn't send data");
    let (amt, src) = socket.recv_from(&mut buf).expect("Failed to receive data"); // TODO relook this expect?
    let buf = &mut buf[..amt];
    println!("Received response from {}", src);
    println!("Data: {}", String::from_utf8_lossy(buf));
}
