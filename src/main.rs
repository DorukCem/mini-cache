use const_format::concatcp;
use tokio::net::{TcpListener, TcpStream};
mod decoder;

const PORT: &'static str = "8012";
const ADDRESS: &'static str = concatcp!("127.0.0.1:", PORT);

async fn start_tcp_server() {
    let listener = TcpListener::bind(ADDRESS).await.unwrap();

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _addr) = listener.accept().await.expect("couldn't get client");
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(socket: TcpStream) {
    let mut decoder = decoder::Decoder::new(socket);
    loop {
        let command = decoder.decode().await;
        println!("Command recieved was {:?}", command)
    }

    // loop {
    //     let mut buf = vec![0; 1024];

    //     let n = socket
    //         .read(&mut buf)
    //         .await
    //         .expect("failed to read data from socket");

    //     if n == 0 {
    //         return;
    //     }

    //     let message_string = core::str::from_utf8(&buf[0..n]).expect("These bytes cannot be converted to UTF-8");
    //     println!("{:?}", message_string);

    //     socket
    //         .write_all(b"+PONG\r\n")
    //         .await
    //         .expect("failed to write data to socket");
    // }
}

#[tokio::main]
async fn main() {
    start_tcp_server().await;
}

#[cfg(test)]
mod tests {}
