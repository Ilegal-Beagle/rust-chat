// network.rs
use std::{
    error::Error,
    io::{BufReader, prelude::*},
    net::{SocketAddr, TcpStream},
    str,
    sync::{Arc, Mutex},
    time::Duration,
    collections::HashMap,
};

use crate::message::MessageType;

pub fn try_connect(address: &SocketAddr, timeout: Duration) -> bool {
    match TcpStream::connect_timeout(address, timeout) {
        Ok(_) => true,
        Err(_) => false,
    }
}

// gets message from TCP, converts it to Message and returns it
pub fn get_message(reader: &mut BufReader<TcpStream>) -> Result<MessageType, Box<dyn Error>> {
    let mut buf = Vec::new();
    reader.read_until(b'\n', &mut buf)?;
    let message = str::from_utf8(&buf)?;

    let deserialized_msg: MessageType = serde_json::from_str(&message)?;
    Ok(deserialized_msg)
}

// sends message via TCP, takes in a MessageType, serializes it and sends it
pub fn send_message(stream: &mut TcpStream, message: MessageType) -> Result<(), Box<dyn Error>> {
    let mut serialized_msg = serde_json::to_string(&message)?;
    serialized_msg.push_str("\n");
    stream.write_all(serialized_msg.as_bytes())?;

    Ok(())
}

pub fn send_to_clients(
    clients: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    message: &str,
) -> Result<(), Box<dyn Error + 'static>> {
    Ok({
        let mut clients = match clients.lock() {
            Ok(c) => c,
            Err(poisoned) => poisoned.into_inner(),
        };

        for (_, stream) in clients.iter_mut() {
            stream.write_all(message.as_bytes())?;
        }
    })
}