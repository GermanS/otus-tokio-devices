use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let socket = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (socket, _) = socket.accept().await.unwrap();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                println!("Error handling connection: {}", e);
            }
        });
    }
}
async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0; 32];

    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => return Ok(()),
            Ok(n) => n,
            Err(e) => return Err(e.into()),
        };

        let recieved = String::from_utf8_lossy(&buf[..n]);
        println!("Received: {}", recieved);
    }
}
