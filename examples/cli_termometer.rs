use std::{io::Write, net::TcpStream};

use otus_tokio_devices::temperature::Temperature;
use otus_tokio_devices::termometer::Termometer;

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("localhost:8080").expect("Unable to connect");

    loop {
        let temperature = rand::random::<f32>() * 100.0;

        let termometer = Termometer::new(Temperature::new(temperature));

        println!("Sending {}", termometer);

        stream.write_all(termometer.to_string().as_bytes()).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
