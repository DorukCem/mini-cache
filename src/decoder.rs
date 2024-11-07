use tokio::{io::AsyncReadExt, net::TcpStream};

#[derive(Debug)]
enum CommandType {
    Get,
    Set,
}

#[derive(Debug)]
struct Header {
    command_name: CommandType,
    key: String,
    flags: u16,
    exptime: i128,
    byte_count: u128,
    no_reply: bool,
}

#[derive(Debug)]
pub struct Command {
    header: Header,
    payload: String,
}

#[derive(Debug)]
struct ParseError;

struct ConnectionBuffer {
    connection: TcpStream,
    buffer: String,
}

impl ConnectionBuffer {
    async fn read_header(&mut self) -> String {
        let data = self.read_until_delimeter().await;
        data
    }

    async fn read_until_delimeter(&mut self) -> String {
        while !self.buffer.contains("\r\n") {
            let mut data = vec![0; 1024];

            let n = self
                .connection
                .read(&mut data)
                .await
                .expect("failed to read data from socket");

            // Socket closed
            if n == 0 {
                return "".to_string(); // TODO turn this in a result
            }

            self.buffer.push_str(
                core::str::from_utf8(&data[0..n])
                    .expect("These bytes cannot be converted to UTF-8"),
            );
        }

        let (header, rest) = split_once(&self.buffer, "\r\n");
        self.buffer = rest;
        return header;
    }

    async fn read_payload(&mut self, byte_count: u128) -> String {
        let byte_count = byte_count + 2; //The extra \r\n is not part of the byte count inside the header field

        if (self.buffer.len() as u128) < byte_count {
            let mut data = vec![0; 1024];

            let n = self
                .connection
                .read(&mut data)
                .await
                .expect("failed to read data from socket");

            // Socket closed
            if n == 0 {
                return "".to_string(); // TODO turn this in a result
            }

            self.buffer.push_str(
                core::str::from_utf8(&data[0..n])
                    .expect("These bytes cannot be converted to UTF-8"),
            );
        }
        let buffer = self.buffer.clone();
        let (payload, rest) = buffer.split_at(byte_count as usize);
        self.buffer = rest.to_string();
        payload.to_string()
    }
}

pub struct Decoder {
    connection: ConnectionBuffer,
}

impl Decoder {
    pub fn new(connection: TcpStream) -> Self {
        return Self {
            connection: ConnectionBuffer {
                connection: connection,
                buffer: String::new(),
            },
        };
    }

    pub async fn decode(&mut self) -> Command {
        let header: String = self.connection.read_header().await;
        let header = parse_header(header).unwrap(); // TODO
        let payload = self.connection.read_payload(header.byte_count).await;
        let payload = parse_payload(payload, header.byte_count).unwrap();
        let command = Command { header, payload };
        command
    }
}

fn parse_payload(mut payload: String, byte_count: u128) -> Result<String, ParseError> {
    if payload.ends_with("\r\n") {
        payload.truncate(byte_count as usize);
        return Ok(payload);
    }

    return Err(ParseError);
}

fn parse_header(header: String) -> Result<Header, ParseError> {
    let keywords: Vec<_> = header.split_whitespace().collect();
    let command = keywords.get(0).ok_or(ParseError)?;
    let command = match command {
        &"get" => CommandType::Get,
        &"set" => CommandType::Set,
        _ => return Err(ParseError),
    };
    let key = keywords.get(1).ok_or(ParseError)?.to_string();
    let flags = match keywords.get(2).ok_or(ParseError)?.parse::<u16>() {
        Ok(value) => value,
        Err(_) => return Err(ParseError),
    };
    let exptime = match keywords.get(3).ok_or(ParseError)?.parse::<i128>() {
        Ok(value) => value,
        Err(_) => return Err(ParseError),
    };
    let byte_count = match keywords.get(4).ok_or(ParseError)?.parse::<u128>() {
        Ok(value) => value,
        Err(_) => return Err(ParseError),
    };
    let _no_reply = keywords.get(5); // TODO

    return Ok(Header {
        command_name: command,
        key,
        flags,
        exptime,
        byte_count,
        no_reply: false,
    });
}

fn split_once(in_string: &str, pat: &str) -> (String, String) {
    let mut splitter = in_string.splitn(2, pat);
    let first = splitter.next().unwrap().to_string();
    let second = splitter.next().unwrap().to_string();
    (first, second)
}
