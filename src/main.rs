use const_format::concatcp;
use decoder::{Command, GetCommand, StorageCommand};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use tokio::net::{TcpListener, TcpStream};
mod decoder;

const PORT: &'static str = "8012";
const ADDRESS: &'static str = concatcp!("127.0.0.1:", PORT);

struct DbEntry {
    value: String,
    flags: u16,
    byte_count: u128,
    valid_until: Option<SystemTime>,
}

struct Database {
    map: Mutex<HashMap<String, DbEntry>>,
}

async fn start_tcp_server() {
    let listener = TcpListener::bind(ADDRESS).await.unwrap();

    let db = Arc::new(Database {
        map: Mutex::new(HashMap::new()),
    });

    loop {
        let db_ref = Arc::clone(&db);
        let (socket, _addr) = listener.accept().await.expect("couldn't get client");
        tokio::spawn(async move {
            handle_connection(socket, db_ref).await;
        });
    }
}

async fn handle_connection(socket: TcpStream, db: Arc<Database>) {
    let mut decoder = decoder::Decoder::new(socket);
    loop {
        let command = decoder.decode().await;
        let response = match command {
            Ok(command) => {
                println!("Command recieved was {:?}", command);
                execute_command(command, &db)
            }
            Err(error) => match error {
                decoder::DecodeError::ConnectionClosed => {
                    println!("connection closed!");
                    return;
                }
                decoder::DecodeError::ParseError(parse_error) => match parse_error {
                    decoder::ParseError::InvalidFormat(client_error) => {
                        format!("CLIENT_ERROR {}\r\n", client_error)
                    }
                    decoder::ParseError::UnknownCommand(_) => "ERROR\r\n".to_owned(),
                },
            },
        };
        println!("Created response: {}", response);
        decoder.send(response).await;
    }
}

fn execute_command(command: Command, db: &Database) -> String {
    let response = match command {
        Command::Set(set_command) => handle_set(set_command, db),
        Command::Get(get_command) => handle_get(get_command, db),
    };

    return response;
}

fn handle_get(command: GetCommand, db: &Database) -> String {
    let mut map = db.map.lock().unwrap();
    match map.get(&command.key) {
        Some(entry) => {
            if let Some(valid_until) = entry.valid_until {
                println!("valid until:{:?}  sytem_time {:?}", valid_until, SystemTime::now());

                if valid_until < SystemTime::now() {
                    // Key expired
                    map.remove(&command.key);
                    return "END\r\n".to_string()
                }
            }

            format!(
                "VALUE {} {} {}\r\n{}\r\nEND\r\n",
                command.key, entry.flags, entry.byte_count, entry.value
            )
        }
        None => "END\r\n".to_string(),
    }
}

fn handle_set(command: StorageCommand, db: &Database) -> String {
    if command.exptime < 0 {
        // If a negative value is given the item is immediately expired.
        return "STORED\r\n".to_string();
    }

    let valid_until = if command.exptime != 0 {
        Some(
            SystemTime::now()
                + Duration::new(
                    command
                        .exptime
                        .try_into()
                        .expect("This should always fit since we discard negatives"),
                    0,
                ),
        )
    } else {
        None
    };

    let mut map = db.map.lock().unwrap();

    let entry = DbEntry {
        value: command.payload,
        flags: command.flags,
        byte_count: command.byte_count,
        valid_until,
    };
    map.insert(command.key, entry);

    return "STORED\r\n".to_string();
}

#[tokio::main]
async fn main() {
    start_tcp_server().await;
}

#[cfg(test)]
mod tests {}
