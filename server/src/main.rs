use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use tftp_libs::{extract_message, Message};

const SERVER_HOST: &str = "127.0.0.1:69";

fn main() {
    let socket = UdpSocket::bind(SERVER_HOST).expect("Failed to bind to udp socket");
    println!("Started TFTP sever ...");

    let mut buf = [0; 512];
    let mut clients = HashMap::new(); //TODO: use a shared data structure/registry

    loop {
        let (amt, src) = socket.recv_from(&mut buf).expect("Failed to receive data"); // TODO relook this expect?
        let buf = &mut buf[..amt];
        println!("Received {} bytes from {}", amt, src);
        clients.insert(src, "connected");
        handle_request(&socket, src, buf);
    }
}

fn handle_request(udp_socket: &UdpSocket, source_address: SocketAddr, buffer: &[u8]) {
    let message = extract_message(buffer);
    match message {
        Message::ReadRequest { file_name, mode } => {
            //TODO: check if file exists
            println!("received request to read {}", file_name);
        }
        Message::WriteRequest { file_name, mode } => {
            println!("received request to write {}", file_name)
        }
        Message::Data {
            block_number,
            data,
            length,
        } => {
            println!(
                "received data of length {} for block {}",
                length, block_number
            )
        }
        Message::Ack { block_number } => {
            println!("received ack of block {}", block_number)
        }
        Message::Error {
            error_code,
            error_message,
        } => {
            println!("received error message :{}", error_message)
        }
    }
}
