use std::net::TcpListener;

fn start_tcp_server() {
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Cannot bind to port");

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    println!("Connection established!");
}

fn main() {
    start_tcp_server();
}

#[cfg(test)]
mod tests {
    use super::*;
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
            TcpStream::connect_timeout(&server_address.parse().unwrap(), Duration::from_secs(5));

        assert!(
            connection.is_ok(),
            "Failed to connect to the TCP server on port 6379"
        );
    }
}
