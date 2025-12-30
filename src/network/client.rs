use crate::{
    message::MessageType,
    network::{helpers, server},
};
use std::{error::Error, net::SocketAddr, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader, split},
    net::TcpStream,
    sync::mpsc::{Receiver, Sender, channel},
};

pub struct NetworkClient {
    tx_ui: Sender<MessageType>,
    rx_net: Receiver<MessageType>,
    _handle: tokio::task::JoinHandle<()>,
}

impl NetworkClient {
    pub fn connect(socket: SocketAddr, rt_handle: &tokio::runtime::Handle) -> Self {
        let (tx_ui, rx_ui) = channel::<MessageType>(128);
        let (tx_net, rx_net) = channel::<MessageType>(128);

        // try to connect to a server
        let handle = rt_handle.spawn(async move {
            match TcpStream::connect(&socket).await {
                Ok(stream) => {
                    // make the client
                    if let Err(e) = client(stream, tx_net, rx_ui).await {
                        eprintln!("Client error: {}", e);
                    }
                }
                Err(_) => {
                    // Start server, then start client
                    tokio::spawn(async move {
                        if let Err(e) = server::server(&socket).await {
                            eprintln!("Server error: {}", e);
                        }
                    });

                    tokio::time::sleep(Duration::from_millis(500)).await;

                    // Create client
                    let stream = TcpStream::connect(&socket).await.unwrap();
                    if let Err(e) = client(stream, tx_net, rx_ui).await {
                        eprintln!("Client error: {}", e);
                    }
                }
            }
        });

        Self {
            tx_ui: tx_ui,
            rx_net: rx_net,
            _handle: handle,
        }
    }

    // send messages to the network side
    pub fn send(&self, message: MessageType, rt: &tokio::runtime::Handle) {
        let tx = self.tx_ui.clone();
        rt.spawn(async move {
            if let Err(e) = tx.send(message).await {
                eprintln!("Error sending message to the network: {e}");
            }
        });
    }

    pub fn recv(&mut self) -> Option<MessageType> {
        match self.rx_net.try_recv() {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }
}

pub async fn client(
    // socket: &SocketAddr,
    stream: TcpStream,
    tx_net: Sender<MessageType>,
    mut rx_ui: Receiver<MessageType>,
) -> Result<(), Box<dyn Error>> {
    println!("client created: {:?}", stream);
    let (reader, mut writer) = split(stream);
    let mut reader = BufReader::new(reader);
    let mut buf = String::new();

    loop {
        tokio::select! {

            // recieve from server
            result = reader.read_line(&mut buf) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        let deserialized_msg: MessageType = match serde_json::from_str(&buf) {
                            Ok(msg) => {msg},
                            Err(e) => {
                                eprintln!("failed to deserialize msg: {}", e);
                                buf.clear();
                                continue;
                            },
                        };

                        tx_net.send(deserialized_msg).await?;

                        buf.clear();
                    },
                    Err(e) => {
                        eprintln!("Network read error: {}", e);
                    },
                }
            }

            // recieve from UI
            msg_opt = rx_ui.recv() => {
                match msg_opt {
                    Some(msg) => {
                        // Successfully received a message from the UI task
                        if let Err(e) =  helpers::send_message(&mut writer, msg).await {
                            eprintln!("error sending message to server: {}", e);
                        };
                    }
                    None => {
                        // The UI's Sender has been dropped, so the client should shut down
                        println!("UI task disconnected. Shutting down client.");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
