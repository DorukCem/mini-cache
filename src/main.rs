use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use const_format::concatcp;

const PORT: &'static str = "7879";
const ADDRESS: &'static str = concatcp!("127.0.0.1:", PORT);

async fn start_tcp_server() {
    let listener = TcpListener::bind(ADDRESS).await.unwrap();

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let mut buf = vec![0; 1024];

    loop {
        let n = socket
            .read(&mut buf)
            .await
            .expect("failed to read data from socket");

        if n == 0 {
            return;
        }
        
        socket
            .write_all(&buf[0..n])
            .await
            .expect("failed to write data to socket");
    }
}

#[tokio::main]
async fn main() {
    start_tcp_server().await;
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
