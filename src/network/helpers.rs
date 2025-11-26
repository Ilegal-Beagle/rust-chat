// network.rs
use std::{
    error::Error,
    net::{SocketAddr},
    collections::HashMap,
    sync::Arc,
};
use tokio::{
    io::{
        AsyncWrite,
        AsyncWriteExt,
        WriteHalf,
    },
    net::{TcpStream},
    sync::Mutex,
};

use crate::message::{MessageType};

// sends message via TCP, takes in a MessageType, serializes it and sends it
pub async fn send_message<W>(writer: &mut W, msg: MessageType) -> Result<(), Box<dyn Error>>
where W: AsyncWrite + Unpin, {
    let serialized = serde_json::to_string(&msg)?;
    writer.write_all(serialized.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}

pub async fn send_to_clients(
    clients: &mut Arc<Mutex<HashMap<SocketAddr, WriteHalf<TcpStream>>>>,
    msg: MessageType,
) -> Result<(), Box<dyn Error + 'static>> {
    Ok({
        let mut clients = clients.lock().await;

        for (socket, stream) in clients.iter_mut() {
            match send_message(stream, msg.clone()).await {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Error sending message to client {:?}: {}", socket, e);
                },
            };
        }
    })
}