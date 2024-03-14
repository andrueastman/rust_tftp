use std::io;
use std::net::{UdpSocket};
use tftp_libs::{extract_message, Message, send_tftp_message};

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

fn get_file(udp_socket: &UdpSocket){
    println!("Download mode:");

    let file_name = get_file_name();
    // send a read request with the file name
    send_tftp_message(udp_socket, Message::ReadRequest {
        file_name: file_name.trim().to_string(),
        mode: "default".to_string()//TODO: implement mode
    }, SERVER_HOST);

    let mut buffer = [0; 520];
    loop {
        let (amt, src) = udp_socket.recv_from(&mut buffer).expect("Failed to receive data"); // TODO relook this expect?
        let buf = &mut buffer[..amt];
        println!("Received {} bytes from {}", amt, src);
        let completed = handle_request(&udp_socket, buf);
        if completed {
            println!("Download Complete");
            println!("*****************************************");
            break;
        }
    }
}

fn put_file(udp_socket: &UdpSocket){
    println!("Upload mode:");
    let file_name = get_file_name();
    // send a read request with the file name
    send_tftp_message(udp_socket, Message::WriteRequest {
        file_name: file_name.to_string(),
        mode: "default".to_string()//TODO: implement mode
    }, SERVER_HOST);

    //TODO upload the file
}

fn get_file_name<'a>() -> String{
    println!("Enter file name: ");
    let mut file_name = String::new();
    io::stdin()
        .read_line(&mut file_name)
        .expect("Failed to read line");

    file_name.trim().to_string()
}

fn handle_request(udp_socket: &UdpSocket, buffer: &[u8]) -> bool {
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

            println!("Data: {}", String::from_utf8_lossy(data));

            //TODO write the contents to a file
            send_tftp_message(udp_socket, Message::Ack { block_number }, SERVER_HOST);
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
            println!("received error code :{}", error_code);
            true
        }
    }
}
