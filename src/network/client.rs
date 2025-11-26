use std::{
    error::Error,
};
use crate::{
    message::MessageType, network::helpers,
};
use tokio::{
    io::{
        AsyncBufReadExt,
        BufReader,
        split,
    },
    net::{TcpStream},
    sync::mpsc::{Receiver, Sender},
};

pub async fn client(
    // socket: &SocketAddr,
    stream: TcpStream,
    tx_net: Sender<MessageType>,
    mut rx_ui: Receiver<MessageType>
) -> Result<(), Box<dyn Error>> {

    // let stream = TcpStream::connect(&socket).await?;
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
                        match helpers::send_message(&mut writer, msg).await {
                            Ok(_) => {println!("message sent")},
                            Err(e) => {eprintln!("error sending message to server: {}", e)},
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