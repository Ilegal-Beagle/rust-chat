use std::{
    io::{BufReader},
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
    let mut reader = BufReader::new(stream.try_clone()?);
    stream.set_read_timeout(Some(Duration::new(3,0)))?;
    println!("connected to server.");

    loop {
        // receive repsonse from server and send it to UI
        if let Ok(msg) = get_message(&mut reader) {
            match msg {
                MessageType::Message(_) => println!("message"),
                MessageType::Notification(_) => println!("notif"),
                MessageType::UserList(_) => println!("user list"),
                MessageType::Disconnect(..) => println!("disconnect"),
                MessageType::Handshake(_) => println!("handshake"),
            }
            tx_net.send(msg)?;
        }

        // get client message from UI
        if let Ok(msg) = rx_ui.try_recv() {
            send_message(&mut stream, msg)?;
        }
    }


}