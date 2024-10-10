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
            .write_all(b"+PONG\r\n")
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
}
