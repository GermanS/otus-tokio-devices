use std::{io::Write, net::TcpStream};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("localhost:8080").expect("Unable to connect");

    loop {
        let message = format!("{:.3} C", rand::random::<f32>() * 100.0);

        println!("Sending temperature {}", message);

        stream.write_all(message.as_bytes()).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
