use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug)]
pub enum Command {
    Set(SetCommand),
    Get(GetCommand),
}

#[derive(Debug)]
pub struct SetCommand {
    pub key: String,
    pub flags: u16,
    pub exptime: i128,
    pub byte_count: u128,
    pub no_reply: bool,
    pub payload: String,
}

#[derive(Debug)]
pub struct GetCommand {
    pub key: String,
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
        let command = parse_header(header).unwrap(); // TODO
        match command {
            Command::Set(set_command) => {
                let payload = self.connection.read_payload(set_command.byte_count).await;
                let payload = parse_payload(payload, set_command.byte_count).unwrap();

                return Command::Set(SetCommand {
                    payload: payload,
                    ..set_command
                });
            }
            Command::Get(_) => return command,
        }
    }

    pub async fn send(&mut self, response: String) -> () {
        let bytes = response.as_bytes();
        self.connection
            .connection
            .write_all(bytes)
            .await
            .expect("failed to write data to socket");
    }
}

fn parse_payload(mut payload: String, byte_count: u128) -> Result<String, ParseError> {
    if payload.ends_with("\r\n") {
        payload.truncate(byte_count as usize);
        return Ok(payload);
    }

    return Err(ParseError);
}

fn parse_header(header: String) -> Result<Command, ParseError> {
    let keywords: Vec<_> = header.split_whitespace().collect();
    let command = keywords.get(0).ok_or(ParseError)?;
    let key = keywords.get(1).ok_or(ParseError)?.to_string();
    match command {
        &"get" => return Ok(Command::Get(GetCommand { key: key })),
        &"set" => {
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

            return Ok(Command::Set(SetCommand {
                key,
                flags,
                exptime,
                byte_count,
                no_reply: false,
                payload: "".to_owned(), // TODO
            }));
        }
        _ => return Err(ParseError),
    };
}

fn split_once(in_string: &str, pat: &str) -> (String, String) {
    let mut splitter = in_string.splitn(2, pat);
    let first = splitter.next().unwrap().to_string();
    let second = splitter.next().unwrap().to_string();
    (first, second)
}
