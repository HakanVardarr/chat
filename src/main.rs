use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast::{channel, Sender},
};

#[derive(Debug, Clone)]
pub struct Client {
    addr: SocketAddr,
    id: String,
}

impl Client {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            id: human_id::id("", true),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:3000";
    let listener = TcpListener::bind(addr).await?;
    let (sender, _reciever) = channel::<Message>(1024);

    loop {
        let (stream, addr) = listener.accept().await?;
        let client = Client::new(addr);
        println!("User connected: {}", client.id);

        let sender = sender.clone();

        tokio::spawn(handle_connection(stream, sender, client));
    }
}

#[derive(Debug, Clone)]
struct Message {
    from: Client,
    payload: String,
}

async fn handle_connection(
    mut stream: TcpStream,
    sender: Sender<Message>,
    client: Client,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 1024];
    let mut reciever = sender.subscribe();

    loop {
        tokio::select! {
            Ok(message) = reciever.recv() => {
                if message.from.addr != client.addr {
                    stream.write(message.payload.as_bytes()).await?;
                }


            }
            Ok(bytes_read) = stream.read(&mut buf) => {
                if bytes_read == 0 {
                    break;
                }

                let payload = String::from_utf8((&buf[..bytes_read]).to_vec())?;
                sender.send(Message {from: client.clone(), payload})?;
            }
        };
    }

    Ok(())
}
