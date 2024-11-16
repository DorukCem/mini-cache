use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug)]
pub enum Command {
    Get(GetCommand),
    Set(StorageCommand),
    Add(StorageCommand),
    Replace(StorageCommand),
    Prepend(StorageCommand),
    Append(StorageCommand),
}

#[derive(Debug)]
pub struct StorageCommand {
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
pub enum ParseError {
    InvalidFormat(String),
    UnknownCommand(String),
}
#[derive(Debug)]
pub struct ConnectionClosedError {}

pub enum DecodeError {
    ConnectionClosed,
    ParseError(ParseError),
}
struct ConnectionBuffer {
    connection: TcpStream,
    buffer: String,
}

impl ConnectionBuffer {
    async fn read_header(&mut self) -> Result<String, ConnectionClosedError> {
        let data = self.read_until_delimeter().await;
        data
    }

    async fn read_until_delimeter(&mut self) -> Result<String, ConnectionClosedError> {
        while !self.buffer.contains("\r\n") {
            let mut data = vec![0; 1024];

            let n = self
                .connection
                .read(&mut data)
                .await
                .expect("failed to read data from socket");

            // Socket closed
            if n == 0 {
                return Err(ConnectionClosedError {});
            }

            self.buffer.push_str(
                core::str::from_utf8(&data[0..n])
                    .expect("These bytes cannot be converted to UTF-8"),
            );
        }

        let (header, rest) = split_once(&self.buffer, "\r\n");
        self.buffer = rest;
        return Ok(header);
    }

    async fn read_payload(&mut self, byte_count: u128) -> Result<String, ConnectionClosedError> {
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
                return Err(ConnectionClosedError {});
            }

            self.buffer.push_str(
                core::str::from_utf8(&data[0..n])
                    .expect("These bytes cannot be converted to UTF-8"),
            );
        }
        let buffer = self.buffer.clone();
        let (payload, rest) = buffer.split_at(byte_count as usize);
        self.buffer = rest.to_string();
        Ok(payload.to_string())
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

    pub async fn decode(&mut self) -> Result<Command, DecodeError> {
        let header = self.connection.read_header().await;
        let header = match header {
            Ok(contents) => contents,
            Err(_conection_closed) => return Err(DecodeError::ConnectionClosed),
        };
        let parse_result = parse_header(header);
        match parse_result {
            Ok(command) => match command {
                Command::Set(set_command) => {
                    return self
                        .parse_storage_command_payload(set_command)
                        .await
                        .map(|x| Command::Set(x))
                }
                Command::Get(_) => return Ok(command),
                Command::Add(add_command) => {
                    return self
                        .parse_storage_command_payload(add_command)
                        .await
                        .map(|x| Command::Add(x))
                }
                Command::Replace(replace_command) => {
                    return self
                        .parse_storage_command_payload(replace_command)
                        .await
                        .map(|x| Command::Replace(x))
                }
                Command::Prepend(prepend_command) =>  {
                    return self
                        .parse_storage_command_payload(prepend_command)
                        .await
                        .map(|x| Command::Prepend(x))
                }
                Command::Append(append_command) =>  {
                    return self
                        .parse_storage_command_payload(append_command)
                        .await
                        .map(|x| Command::Append(x))
                }
            },
            Err(parse_error) => Err(DecodeError::ParseError(parse_error)),
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

    async fn parse_storage_command_payload(
        &mut self,
        command: StorageCommand,
    ) -> Result<StorageCommand, DecodeError> {
        let payload = self.connection.read_payload(command.byte_count).await;
        let payload = match payload {
            Ok(value) => value,
            Err(_error) => return Err(DecodeError::ConnectionClosed),
        };

        let payload = match parse_payload(payload, command.byte_count) {
            Ok(value) => value,
            Err(error) => return Err(DecodeError::ParseError(error)),
        };

        return Ok(StorageCommand {
            payload: payload,
            ..command
        });
    }
}

fn parse_payload(mut payload: String, byte_count: u128) -> Result<String, ParseError> {
    if payload.ends_with("\r\n") {
        payload.truncate(byte_count as usize);
        return Ok(payload);
    }

    return Err(ParseError::InvalidFormat(
        "Expected \r\n at the end of string".to_string(),
    ));
}

fn parse_header(header: String) -> Result<Command, ParseError> {
    // ? I think we cannot detect missing \r\n in header because we must read untill we see \r\n
    // ? It is up to the client to not mess this up. 

    let keywords: Vec<_> = header.split_whitespace().collect();
    let command = keywords
        .get(0)
        .ok_or(ParseError::InvalidFormat("Missing command".to_string()))?;
    let key = keywords
        .get(1)
        .ok_or(ParseError::InvalidFormat("Missing key".to_string()))?
        .to_string();

    match command.to_lowercase().as_str() {
        "get" => Ok(Command::Get(GetCommand { key })),
        "set" | "add" | "replace" | "append" | "prepend" => {
            let storage_command = parse_storage_command(&keywords, key)?;
            match *command {
                "set" => Ok(Command::Set(storage_command)),
                "add" => Ok(Command::Add(storage_command)),
                "replace" => Ok(Command::Replace(storage_command)),
                "append" => Ok(Command::Append(storage_command)),
                "prepend" => Ok(Command::Prepend(storage_command)),
                _ => unreachable!(),
            }
        }
        _ => Err(ParseError::UnknownCommand(command.to_string())),
    }
}

fn parse_storage_command(keywords: &[&str], key: String) -> Result<StorageCommand, ParseError> {
    let flags = parse_field::<u16>(keywords, 2, "flags")?;
    let exptime = parse_field::<i128>(keywords, 3, "exptime")?;
    let byte_count = parse_field::<u128>(keywords, 4, "byte count")?;
    let no_reply = keywords.get(5).is_some(); // If there's a 6th keyword, assume "no_reply"

    Ok(StorageCommand {
        key,
        flags,
        exptime,
        byte_count,
        no_reply,
        payload: "".to_owned(),
    })
}

fn parse_field<T: std::str::FromStr>(
    keywords: &[&str],
    index: usize,
    field_name: &str,
) -> Result<T, ParseError> {
    keywords
        .get(index)
        .ok_or(ParseError::InvalidFormat(format!(
            "{} is missing",
            field_name
        )))?
        .parse::<T>()
        .map_err(|_parse_number_errror| ParseError::InvalidFormat(format!("Expected a valid {} value", field_name)))
}

fn split_once(in_string: &str, pat: &str) -> (String, String) {
    let mut splitter = in_string.splitn(2, pat);
    let first = splitter.next().unwrap().to_string();
    let second = splitter.next().unwrap().to_string();
    (first, second)
}
