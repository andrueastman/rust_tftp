use std::io::{BufRead, Seek, SeekFrom};
use std::net::{SocketAddr, UdpSocket};
use tftp_libs::{
    extract_message, get_read_file_info, send_error_message, send_tftp_message, Message,
    SessionRegistry, TftpSessionInfo,
};

const SERVER_HOST: &str = "127.0.0.1:69";
fn main() {
    let socket = UdpSocket::bind(SERVER_HOST).expect("Failed to bind to udp socket");
    println!("Started TFTP sever ...");

    let mut session_registry = SessionRegistry::new();
    let mut buf = [0; 512];
    loop {
        let receive_result = socket.recv_from(&mut buf);
        if receive_result.is_err() {
            println!("Failed to receive data");
            continue;
        }
        let (amt, src) = receive_result.unwrap();
        let received_buffer = &mut buf[..amt];
        println!("Received {} bytes from {}", amt, src);
        session_registry.register(src, TftpSessionInfo::new()); //try to register the session incase its new.
        handle_request(&socket, src, received_buffer, &mut session_registry);
    }
}

fn handle_request(
    udp_socket: &UdpSocket,
    source_address: SocketAddr,
    buffer: &[u8],
    session_registry: &mut SessionRegistry,
) {
    let message = extract_message(buffer);
    let session_info = session_registry
        .get_session(source_address)
        .expect("Unable to get session information");
    match message {
        Message::ReadRequest { file_name, mode } => {
            println!("received request to read {} with mode {}", file_name, mode);
            // Try to find the file
            let file_result = get_read_file_info(file_name.clone());
            let (reader, file_length) = match file_result {
                Ok((reader, length)) => (reader, length),
                Err(error) => {
                    send_error_message(error, udp_socket, &source_address.to_string());
                    return; // nothing else to do here
                }
            };

            // update the session information
            session_info.file_name = String::from(file_name);
            session_info.reader = Some(reader);
            session_info.block_count = ((file_length / 512) + 1u64) as usize;

            let contents = session_info
                .reader
                .as_mut()
                .expect("failed to get reader")
                .fill_buf()
                .expect("Unable to read file contents");
            let block_number = 1;

            // Send back the first chunk
            send_tftp_message(
                udp_socket,
                Message::Data {
                    block_number,
                    data: contents[0..contents.len()].as_ref(),
                    length: contents.len(),
                },
                &source_address.to_string(),
            );

            println!("Sent back first block of {} bytes", contents.len());
        }
        Message::WriteRequest { file_name, mode } => {
            println!("received request to write {} with mode {}", file_name, mode);
            let block_number = 0;
            send_tftp_message(
                udp_socket,
                Message::Ack { block_number },
                &source_address.to_string(),
            );
            println!("Sent ack to start upload");
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
            println!("Data: {}", String::from_utf8_lossy(data));
        }
        Message::Ack { block_number } => {
            println!("received ack of block {}", block_number);
            // check if we are done
            if block_number == session_info.block_count as u16 {
                println!(
                    "Received last ack for file name: {}",
                    session_info.file_name
                );
                session_registry.deregister(source_address);
                return;
            }

            println!("Reading next block of file: {}", session_info.file_name);
            let reader = session_info.reader.as_mut().expect("Reader not found");
            reader
                .seek(SeekFrom::Current(512))
                .expect("Unable to seek file"); // move to next block
            let contents = reader.fill_buf().expect("Unable to read file contents");
            let block_number = block_number + 1;

            //TODO send back error if block is out of range
            send_tftp_message(
                udp_socket,
                Message::Data {
                    block_number,
                    data: contents[0..contents.len()].as_ref(),
                    length: contents.len(),
                },
                &source_address.to_string(),
            );

            println!(
                "Sent back block number {} of {} bytes",
                block_number,
                contents.len()
            );
            if contents.len() < 512 {
                println!("Sent back last block to client");
            }
        }
        Message::Error {
            error_code,
            error_message,
        } => {
            eprintln!("received error message :{}", error_message);
            eprintln!("received error code :{}", error_code);
        }
    }
}
