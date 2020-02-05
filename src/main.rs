use std::error::Error;
use std::env;
use tokio::net::UdpSocket;

use rtp2wav::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:34254".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening on: {}", socket.local_addr()?);

    let server = Server {
        socket,
        buf: vec![0u8; 1500],
        to_send: None,
    };

    server.run().await?;

    Ok(())
}