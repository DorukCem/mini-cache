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
                    decoder::ParseError::UnknownCommand(_message) => "ERROR\r\n".to_owned(),
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
        Command::Add(add_command) => handle_add(add_command, db),
        Command::Replace(replace_command) => handle_replace(replace_command, db),
    };

    return response;
}
fn handle_replace(command: StorageCommand, db: &Database) -> String {
    let mut map = db.map.lock().unwrap();

    if is_key_valid(&command.key, &map) {
        insert_entry(
            command.key,
            command.payload,
            command.flags,
            command.byte_count,
            command.exptime,
            &mut map,
        );
        return "STORED\r\n".to_string();
    }

    "NOT_STORED\r\n".to_string()
}

fn handle_add(command: StorageCommand, db: &Database) -> String {
    let mut map = db.map.lock().unwrap();

    if is_key_valid(&command.key, &map) {
        return "NOT_STORED\r\n".to_string();
    }

    insert_entry(
        command.key,
        command.payload,
        command.flags,
        command.byte_count,
        command.exptime,
        &mut map,
    );
    "STORED\r\n".to_string()
}

fn handle_set(command: StorageCommand, db: &Database) -> String {
    let mut map = db.map.lock().unwrap();

    insert_entry(
        command.key,
        command.payload,
        command.flags,
        command.byte_count,
        command.exptime,
        &mut map,
    );
    "STORED\r\n".to_string()
}

fn handle_get(command: GetCommand, db: &Database) -> String {
    let mut map = db.map.lock().unwrap();
    match map.get(&command.key) {
        Some(entry) => {
            if let Some(valid_until) = entry.valid_until {
                if valid_until < SystemTime::now() {
                    // Key expired
                    map.remove(&command.key);
                    return "END\r\n".to_string();
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

fn is_key_valid(key: &str, map: &HashMap<String, DbEntry>) -> bool {
    // The keys are lazlily evaluated and removed
    if let Some(entry) = map.get(key) {
        if let Some(valid_until) = entry.valid_until {
            if valid_until > SystemTime::now() {
                return true;
            }
        }
    }
    false
}

fn calculate_valid_until(exptime: i128) -> Option<SystemTime> {
    if exptime > 0 {
        Some(SystemTime::now() + Duration::new(exptime.try_into().unwrap(), 0))
    } else {
        None
    }
}

fn insert_entry(
    key: String,
    payload: String,
    flags: u16,
    byte_count: u128,
    exptime: i128,
    map: &mut HashMap<String, DbEntry>,
) {
    if exptime < 0 {
        return;
    }

    let valid_until = calculate_valid_until(exptime);
    let entry = DbEntry {
        value: payload,
        flags,
        byte_count,
        valid_until,
    };
    map.insert(key, entry);
}

#[tokio::main]
async fn main() {
    start_tcp_server().await;
}

#[cfg(test)]
mod tests {}
