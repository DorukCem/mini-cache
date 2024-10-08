use std::{io::{BufReader, Write}, net::{TcpListener, TcpStream}};

fn start_tcp_server() {
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Cannot bind to port");

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

fn handle_connection(mut stream: TcpStream){
    let _buf_reader = BufReader::new(&stream);

    let response = "+PONG\r\n";
    stream.write_all(response.as_bytes()).unwrap();
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

        let server_address = "127.0.0.1:6379";
        let connection =
            TcpStream::connect_timeout(&server_address.parse().unwrap(), Duration::from_secs(1));

        assert!(
            connection.is_ok(),
            "Failed to connect to the TCP server on port 6379"
        );
    }


    #[test]
    fn test_ping_pong_response() {
        thread::spawn(|| {
            super::start_tcp_server();
        });

        thread::sleep(Duration::from_secs(1));

        let mut stream = TcpStream::connect("127.0.0.1:6379")
            .expect("Failed to connect to the TCP server on port 6379");

        let ping_message = b"PING\r\n";
        stream.write_all(ping_message).expect("Failed to send PING command");

        let mut buffer = [0; 512];
        let size = stream.read(&mut buffer).expect("Failed to read response from server");

        // Convert the response to a string for validation
        let response = String::from_utf8_lossy(&buffer[..size]);

        // Check if the response matches the expected PONG response
        assert_eq!(response, "+PONG\r\n", "The server did not respond with +PONG\\r\\n");
    }
}
