use std::fs::File;
use std::io;
use std::io::{BufRead, BufWriter, Seek, SeekFrom, Write};
use std::net::UdpSocket;
use tftp_libs::{extract_message, get_read_file_info, send_tftp_message, Message, TftpSessionInfo};

const SERVER_HOST: &str = "127.0.0.1:69";

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:34252").expect("Failed to bind to udp socket");

    println!("Welcome to a simple TFTP client!");
    loop {
        println!("*****************************************");
        println!("Select one of the options to do something");
        println!("1. GET a file from the server");
        println!("2. PUT a file to the server");
        println!("0. Exit");

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        let guess: u32 = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        match guess {
            0 => {
                println!("Sayonara ...");
                break;
            }
            1 => get_file(&socket),
            2 => put_file(&socket),
            _ => {
                println!("Invalid option...");
                continue;
            }
        }
    }
}

fn get_file(udp_socket: &UdpSocket) {
    println!("Download mode:");

    let file_name = get_file_name();
    let mut session_info = TftpSessionInfo::new();
    session_info.file_name = file_name.clone();

    // send a read request with the file name
    send_tftp_message(
        udp_socket,
        Message::ReadRequest {
            file_name: file_name.trim().to_string(),
            mode: "default".to_string(), //TODO: implement mode
        },
        SERVER_HOST,
    );

    do_work(udp_socket, &mut session_info);
}

fn put_file(udp_socket: &UdpSocket) {
    println!("Upload mode:");
    let file_name = get_file_name();

    let file_result = get_read_file_info(file_name.clone());
    let (reader, file_length) = match file_result {
        Ok((reader, length)) => (reader, length),
        Err(error) => {
            eprintln!("Error reading file: {}", error);
            return; // nothing else to do here
        }
    };

    let mut session_info = TftpSessionInfo::new();
    session_info.file_name = file_name.clone();
    session_info.reader = Some(reader);
    session_info.block_count = ((file_length / 512) + 1u64) as usize;

    // send a write request with the file name
    send_tftp_message(
        udp_socket,
        Message::WriteRequest {
            file_name: file_name.to_string(),
            mode: "default".to_string(), //TODO: implement mode
        },
        SERVER_HOST,
    );

    do_work(udp_socket, &mut session_info);
}

fn do_work(udp_socket: &UdpSocket, session_info: &mut TftpSessionInfo) {
    let mut buffer = [0; 520];
    loop {
        let receive_result = udp_socket.recv_from(&mut buffer);
        if receive_result.is_err() {
            println!("Failed to receive data");
            continue;
        }
        let (amt, _) = receive_result.unwrap();
        let buf = &mut buffer[..amt];
        let completed = handle_request(&udp_socket, buf, session_info);
        if completed {
            println!("*****************************************");
            break;
        }
    }
}
fn get_file_name<'a>() -> String {
    println!("Enter file name: ");
    let mut file_name = String::new();
    io::stdin()
        .read_line(&mut file_name)
        .expect("Failed to read line");

    file_name.trim().to_string()
}

fn handle_request(
    udp_socket: &UdpSocket,
    buffer: &[u8],
    session_info: &mut TftpSessionInfo,
) -> bool {
    let message = extract_message(buffer);
    match message {
        Message::ReadRequest { file_name, mode } => {
            // TODO client doesn't need to handle read requests
            println!("received request to read {} with mode {}", file_name, mode);
            panic!("Client received read request")
        }
        Message::WriteRequest { file_name, mode } => {
            // TODO client doesn't need to handle read requests
            println!("received request to write {} with mode {}", file_name, mode);
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
            if block_number == 1 {
                let file =
                    File::create(session_info.file_name.clone()).expect("Error creating file");
                let writer = BufWriter::new(file);
                session_info.writer = Some(writer);
            }
            //write the contents to file
            session_info
                .writer
                .as_mut()
                .expect("Writer not set")
                .write(data)
                .expect("Error writing chunk to file");
            // TODO handle write error and re-request the block??
            send_tftp_message(udp_socket, Message::Ack { block_number }, SERVER_HOST);
            println!("sent back ack for block number {}", block_number);
            if length < 512 {
                println!("Download Complete");
                return true;
            }
            return false;
        }
        Message::Ack { block_number } => {
            println!("received ack of block {}", block_number);
            // check if we are done
            if block_number == session_info.block_count as u16 {
                println!(
                    "Received last ack for file name: {}",
                    session_info.file_name
                );
                println!("Download Complete");
                return true;
            }

            println!("Reading next block of file: {}", session_info.file_name);
            let reader = session_info.reader.as_mut().expect("Reader not found");
            if block_number != 0 {
                reader
                    .seek(SeekFrom::Current(512))
                    .expect("Unable to seek file"); // move to next block
            }
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
                SERVER_HOST,
            );

            println!(
                "Sent back block number {} of {} bytes",
                block_number,
                contents.len()
            );
            if contents.len() < 512 {
                println!("Sent back last block to client");
            }
            false
        }
        Message::Error {
            error_code,
            error_message,
        } => {
            eprintln!("received error message : {}", error_message);
            eprintln!("received error code : {}", error_code);
            true
        }
    }
}
