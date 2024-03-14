use std::fs;
use std::net::{SocketAddr, UdpSocket};
use tftp_libs::{build_message, extract_message, Message};

const SERVER_HOST: &str = "127.0.0.1:69";

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:34252").expect("Failed to bind to udp socket");

    let mut buf = [0; 520];
    let message = Message::ReadRequest {
        file_name: "test.json".to_string(),
        mode: "Hello".to_string(),
    };
    let buffer = build_message(message);
    socket
        .send_to(&buffer, SERVER_HOST)
        .expect("couldn't send data");
    loop {
        let (amt, src) = socket.recv_from(&mut buf).expect("Failed to receive data"); // TODO relook this expect?
        let buf = &mut buf[..amt];
        println!("Received {} bytes from {}", amt, src);
        let completed = handle_request(&socket, src, buf);
        if completed {
            break;
        }
    }
}

fn handle_request(udp_socket: &UdpSocket, source_address: SocketAddr, buffer: &[u8]) -> bool{
    let message = extract_message(buffer);
    match message {
        Message::ReadRequest { file_name, mode } => {
            // TODO client doesn't need to handle read requests
            panic!("Client received read request")
        }
        Message::WriteRequest { file_name, mode } => {
            // TODO client doesn't need to handle read requests
            panic!("Client received write request")
        }
        Message::Data {
            block_number,
            data,
            length,
        } => {
            println!(
                "received data of length {} for block {}",
                length, block_number
            );

            println!(
                "Data: {}",
                String::from_utf8_lossy(data)          );
            let message = Message::Ack { block_number };
            let message_data = build_message(message);
            //TODO write the contents to a file
            udp_socket
                .send_to(&message_data, source_address)
                .expect("Failed to send data");
            println!("sent back ack for block number {}", block_number);
            if length < 512 {
                return true;
            }
            return false;
        }
        Message::Ack { block_number } => {
            println!("received ack of block {}", block_number);
            false
        }
        Message::Error {
            error_code,
            error_message,
        } => {
            println!("received error message :{}", error_message);
            true
        }
    }
}
