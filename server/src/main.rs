use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use tftp_libs::{build_message, extract_message, Message};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek};

const SERVER_HOST: &str = "127.0.0.1:69";

struct ClientInfo {
    file_name: String,
    reader: Option<BufReader<File>>
}

fn main() {
    let socket = UdpSocket::bind(SERVER_HOST).expect("Failed to bind to udp socket");
    println!("Started TFTP sever ...");

    let mut buf = [0; 512];
    let mut clients:HashMap<SocketAddr, ClientInfo> = HashMap::new(); //TODO: use a shared data structure/registry

    loop {
        let receiveResult = socket.recv_from(&mut buf);
        if receiveResult.is_err() {
            println!("Failed to receive data");
            continue;
        }
        let (amt, src) = receiveResult.unwrap();
        let buf = &mut buf[..amt];
        println!("Received {} bytes from {}", amt, src);
        if(!clients.contains_key(&src)) {
            let client_info = ClientInfo {
                file_name: String::new(),
                reader: None
            };
            clients.insert(src, client_info);
        }

        handle_request(&socket, src, buf, &mut clients);
    }
}



fn handle_request(udp_socket: &UdpSocket, source_address: SocketAddr, buffer: &[u8], clients: &mut HashMap<SocketAddr, ClientInfo>) {
    let message = extract_message(buffer);
    match message {
        Message::ReadRequest { file_name, mode } => {

            println!("received request to read {}", file_name);
            let file = File::open(file_name.clone()).expect("Unable to read file with the given file name");
            let mut reader = BufReader::with_capacity(512, file);
            let contents = reader.fill_buf().expect("Unable to read file contents");
            let block_size = if { contents.len() > 512 } { 512 } else { contents.len() };
            let block_number = 1;

            let block_message = Message::Data {
                block_number,
                data: contents[0..block_size].as_ref(),
                length: block_size,
            };
            let message_data = build_message(block_message);
            udp_socket
                .send_to(&message_data, source_address)
                .expect("Failed to send data");

            let client_info = clients.get_mut(&source_address).expect("Unable to get file name");
            client_info.file_name = String::from(file_name);
            client_info.reader = Option::from(reader);
            println!("Sent back first block of {} bytes", block_size);
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
            println!("received ack of block {}", block_number);
            let client_info = clients.get_mut(&source_address).expect("Unable to get file name");
            println!("Reading next block of file: {}", client_info.file_name);
            let reader = client_info.reader.as_mut().expect("Reader not found");
            reader.seek(std::io::SeekFrom::Current(512)).expect("Unable to seek file");
            let contents = reader.fill_buf().expect("Unable to read file contents");
            let block_size = if { contents.len() > 512 } { 512 } else { contents.len() };
            let block_number = block_number + 1;

            //TODO send back error if block is out of range
            let block_message = Message::Data {
                block_number,
                data: contents[0..block_size].as_ref(),
                length: block_size,
            };
            let message_data = build_message(block_message);
            udp_socket
                .send_to(&message_data, source_address)
                .expect("Failed to send data");

            println!("Sent back block number {} of {} bytes", block_number,block_size);
        }
        Message::Error {
            error_code,
            error_message,
        } => {
            println!("received error message :{}", error_message)
        }
    }
}
