use std::{io::Write, net::TcpStream};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();

    loop {
        let power = rand::random::<u8>();

        let message = format!("{} W", power);

        println!("Sent message: {}", message);

        stream.write_all(message.as_bytes()).unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
