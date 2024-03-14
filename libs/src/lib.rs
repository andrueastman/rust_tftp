use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind};
use std::net::{SocketAddr, UdpSocket};

fn build_message(tftp_message: Message) -> Vec<u8> {
    match tftp_message {
        Message::ReadRequest { file_name, mode } => {
            let mut message = vec![0; 2 + file_name.len() + 1 + mode.len() + 1];
            message[0] = 0;
            message[1] = OpCode::Read as u8;
            message[2..file_name.len() + 2].copy_from_slice(file_name.as_bytes());
            message[file_name.len() + 2] = 0;
            message[file_name.len() + 2 + 1..file_name.len() + 2 + 1 + mode.len()]
                .copy_from_slice(mode.as_bytes());
            message[file_name.len() + 2 + 1 + mode.len()] = 0;
            message
        }
        Message::WriteRequest { file_name, mode } => {
            let mut message = vec![0; 2 + file_name.len() + 1 + mode.len() + 1];
            message[0] = 0;
            message[1] = OpCode::Write as u8;
            message[2..file_name.len()].copy_from_slice(file_name.as_bytes());
            message[file_name.len()] = 0;
            message[file_name.len() + 1..file_name.len() + 1 + mode.len()]
                .copy_from_slice(mode.as_bytes());
            message[file_name.len() + 1 + mode.len()] = 0;
            message
        }
        Message::Data {
            block_number,
            data,
            length,
        } => {
            let mut message = vec![0; 4 + length];
            //TODO verify block length is not greater than 512
            message[0] = 0;
            message[1] = OpCode::Data as u8;
            message[2] = (block_number >> 8) as u8;
            message[3] = block_number as u8;
            message[4..4 + length].copy_from_slice(data);
            message
        }
        Message::Ack { block_number } => {
            let mut message = vec![0; 4];
            message[0] = 0;
            message[1] = OpCode::Ack as u8;
            message[2] = (block_number >> 8) as u8;
            message[3] = block_number as u8;
            message
        }
        Message::Error {
            error_code,
            error_message,
        } => {
            let mut message = vec![0; 4 + error_message.len() + 1];
            message[0] = 0;
            message[1] = OpCode::Error as u8;
            message[2] = (error_code >> 8) as u8;
            message[3] = error_code as u8;
            message[4..error_message.len() + 4].copy_from_slice(error_message.as_bytes());
            message[4 + error_message.len()] = 0;
            message
        }
    }
}

pub fn extract_opcode(buffer: &[u8]) -> OpCode {
    //TODO: verify buffer length is at least 2 or exactly 2
    let opcode = (buffer[0] as u16) << 8 | buffer[1] as u16;
    match opcode {
        1 => OpCode::Read,
        2 => OpCode::Write,
        3 => OpCode::Data,
        4 => OpCode::Ack,
        _ => OpCode::Error,
    }
}

pub fn send_tftp_message(udp_socket: &UdpSocket, message: Message, destination: &str) {
    let message_data = build_message(message);
    udp_socket
        .send_to(&message_data, destination)
        .expect("Failed to send data");
}

pub fn extract_message(buffer: &[u8]) -> Message {
    let opcode = extract_opcode(buffer);
    match opcode {
        OpCode::Read => {
            let mut file_name = String::new();
            let mut mode = String::new();
            let mut i = 2;
            while buffer[i] != 0 {
                file_name.push(buffer[i] as char);
                i += 1;
            }
            i += 1;
            while buffer[i] != 0 {
                mode.push(buffer[i] as char);
                i += 1;
            }
            Message::ReadRequest { file_name, mode }
        }
        OpCode::Write => {
            let mut file_name = String::new();
            let mut mode = String::new();
            let mut i = 2;
            while buffer[i] != 0 {
                file_name.push(buffer[i] as char);
                i += 1;
            }
            i += 1;
            while buffer[i] != 0 {
                mode.push(buffer[i] as char);
                i += 1;
            }
            Message::WriteRequest { file_name, mode }
        }
        OpCode::Data => {
            let block_number = (buffer[2] as u16) << 8 | buffer[3] as u16;
            let data = &buffer[4..];
            Message::Data {
                block_number,
                data,
                length: data.len(),
            }
        }
        OpCode::Ack => {
            let block_number = (buffer[2] as u16) << 8 | buffer[3] as u16;
            Message::Ack { block_number }
        }
        OpCode::Error => {
            let error_code = (buffer[2] as u16) << 8 | buffer[3] as u16;
            let mut error_message = String::new();
            let mut i = 4;
            while buffer[i] != 0 {
                error_message.push(buffer[i] as char);
                i += 1;
            }
            Message::Error {
                error_code,
                error_message,
            }
        }
    }
}

pub fn get_read_file_info(file_name: String) -> Result<(BufReader<File>, u64), Error> {
    match File::open(file_name) {
        Ok(file) => {
            let file_length = file.metadata().expect("Unable to read metadata").len();
            let reader = BufReader::with_capacity(512, file);
            Ok((reader, file_length))
        }
        Err(error) => Err(error),
    }
}

pub fn send_error_message(error: Error, udp_socket: &UdpSocket, destination: &str) {
    let message: Message = match error.kind() {
        ErrorKind::NotFound => {
            // send a not found message
            Message::Error {
                error_code: 1,
                error_message: "File not found".to_string(),
            }
        }
        // send back generic error
        error => {
            Message::Error {
                error_code: 0, //unknown
                error_message: error.to_string(),
            }
        }
    };
    send_tftp_message(udp_socket, message, destination);
}

#[derive(Debug)]
pub enum OpCode {
    Read = 1,
    Write = 2,
    Data = 3,
    Ack = 4,
    Error = 5,
}

pub enum Message<'t> {
    ReadRequest {
        file_name: String,
        mode: String,
    }, //name and mode
    WriteRequest {
        file_name: String,
        mode: String,
    }, //name and mode
    Data {
        block_number: u16,
        data: &'t [u8],
        length: usize,
    }, //block number
    Ack {
        block_number: u16,
    }, //block number
    Error {
        error_code: u16,
        error_message: String,
    }, //error code and error message
}

pub struct TftpSessionInfo {
    pub file_name: String,
    pub reader: Option<BufReader<File>>,
    pub writer: Option<BufWriter<File>>,
    pub block_count: usize,
}

impl TftpSessionInfo {
    pub fn new() -> Self {
        TftpSessionInfo {
            file_name: String::new(),
            reader: None,
            writer: None,
            block_count: 0,
        }
    }
}
pub struct SessionRegistry {
    sessions: HashMap<SocketAddr, TftpSessionInfo>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        SessionRegistry {
            sessions: HashMap::new(),
        }
    }

    pub fn register(&mut self, address: SocketAddr, session_info: TftpSessionInfo) {
        if !self.sessions.contains_key(&address) {
            self.sessions.insert(address, session_info);
        }
    }

    pub fn deregister(&mut self, address: SocketAddr) {
        self.sessions.remove(&address);
    }

    pub fn get_session(&mut self, address: SocketAddr) -> Option<&mut TftpSessionInfo> {
        self.sessions.get_mut(&address)
    }
}
