use std::{
    net::{SocketAddr, TcpStream},
    sync::{mpsc::{Sender, Receiver}},
    error::Error,
    time::Duration,
};
use crate::{
    message::MessageType,
    network::helpers::{send_message, get_message},
};

// does the client functions. gets a message from user and sends and reveives it back from server.
pub fn client(
    socket: &SocketAddr,
    rx_ui: Receiver<MessageType>,
    tx_net: Sender<MessageType>,
) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(&socket)?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    println!("connected to server.");

    loop {
        // receive repsonse from server and send it to UI
        if let Ok(msg) = get_message(&mut stream) {
            tx_net.send(msg)?;
            println!("got message from server and sent it to ui");
        }

        // get client message from UI
        if let Ok(msg) = rx_ui.try_recv() {
            send_message(&mut stream, msg)?;
            println!("got messagr from UI and sent it to the server");
        }
    }
}