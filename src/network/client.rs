use std::{
    error::Error,
    net::SocketAddr,
};
use crate::{
    message::MessageType,
    network::helpers::{send_message, get_message},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc::{Sender, Receiver},
};

#[tokio::main]
pub async fn client(
    socket: &SocketAddr,
    tx_net: Sender<MessageType>,
    rx_ui: Receiver<MessageType>
) -> Result<(), Box<dyn Error>> {

    let mut stream = TcpStream::connect(&socket).await?;
    let mut reader = BufReader::new(stream);
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

                        match tx_net.send(deserialized_msg).await {
                            Ok(_) => {},
                            Err(e) => {
                                eprintln!("failed to send to UI: {}", e);
                                panic!();
                            }
                        };

                        buf.clear();
                    },
                    Err(e) => {
                        eprintln!("Network read error: {}", e);
                    },
                }
            }

            // recieve from UI
            msg_opt = rx_ui.recv().await => {
                match msg_opt {
                    Some(msg) => {
                        // Successfully received a message from the UI task
                        if let Err(e) = send_message_async(&mut stream, msg).await {
                            eprintln!("Failed to send message to server: {}", e);
                            break;
                        }
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

async fn send_message_async(stream: &mut TcpStream, msg: MessageType) -> Result<(), Box<dyn Error>> {
    let serialized = serde_json::to_string(&msg)?;
    stream.write_all(serialized.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    Ok(())
}