use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use const_format::concatcp;

const PORT: &'static str = "7879";
const ADDRESS: &'static str = concatcp!("127.0.0.1:", PORT);

fn start_tcp_server() {
    let listener = TcpListener::bind(ADDRESS).expect("Cannot bind to port");
    println!("Server listening on {ADDRESS}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    println!("Connection established!");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request: Vec<_> = buf_reader
        .lines()
        .flatten()
        .take_while(|line| !line.is_empty())
        .collect();

    println!("{:#?}", &request);
    for command in request {
        let response = "+PONG\r\n";
        stream.write_all(response.as_bytes()).unwrap();
    }

}

fn main() {
    start_tcp_server();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_tcp_server_connection() {
        thread::spawn(|| {
            start_tcp_server();
        });

        // Give the server some time to start up
        thread::sleep(Duration::from_secs(1));

        let server_address = ADDRESS;
        let connection =
            TcpStream::connect_timeout(&server_address.parse().unwrap(), Duration::from_secs(1));

        assert!(connection.is_ok(), "Failed to connect to the TCP server");
    }

    #[test]
    fn test_ping_pong_response() {
        thread::spawn(|| {
            super::start_tcp_server();
        });

        thread::sleep(Duration::from_secs(1));

        let mut stream = std::net::TcpStream::connect(ADDRESS).unwrap();
        writeln!(stream, "PING\n").unwrap();
        let mut buffer: [u8; 7] = [0; 7];
        stream.read(&mut buffer).unwrap();
        assert_eq!(buffer.as_slice(), b"+PONG\r\n");
    }
}
