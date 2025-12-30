use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};

use crate::message::MessageType;
use crate::network::helpers::send_to_clients;

#[derive(Debug, Default)]
pub(crate) struct Room {
    pub(crate)name: String,
    pub(crate)client_sockets: Arc<Mutex<HashMap<SocketAddr, WriteHalf<TcpStream>>>>,
    pub(crate)client_names: Arc<Mutex<HashMap<String, String>>>,
    pub(crate)messages: Vec<MessageType>,
}

impl Room {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub async fn broadcast(&self, msg: MessageType) {
        let mut clients_clone = Arc::clone(&self.client_sockets);
        if let Err(e) = send_to_clients(&mut clients_clone, msg).await {
            eprintln!("Broadcast Error: {e}");
        }
    }
}