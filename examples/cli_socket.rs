use std::{io::Write, net::TcpStream};

use otus_tokio_devices::power::Power;
use otus_tokio_devices::socket::Socket;

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();

    loop {
        let power = rand::random::<u8>();

        let socket = Socket::new( Power::new(power as f32) );

        println!("Sent message: {}", socket);

        stream.write_all(socket.to_string().as_bytes()).unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
