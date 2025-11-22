// network.rs
use std::{
    collections::HashMap,
    error::Error,
    io::{BufReader, prelude::*},
    net::{SocketAddr, TcpListener, TcpStream},
    str,
    sync::mpsc::{Receiver, Sender},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::message::MessageType;

pub fn try_connect(address: &SocketAddr, timeout: Duration) -> bool {
    match TcpStream::connect_timeout(address, timeout) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn handle_connection(
    mut stream: TcpStream,
    clients: Arc<Mutex<Vec<TcpStream>>>,
    names: Arc<Mutex<HashMap<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 128];

    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 { break; }
        let message = str::from_utf8(&buf[..bytes_read])?;

        {
            let mut user_list = match names.lock() {
                Ok(c) => c,
                Err(poisoned) => poisoned.into_inner(),
            };
            let deserialized_msg: MessageType = serde_json::from_str(&message)?;
            deserialized_msg.handle(&mut user_list, &clients);
        }

        send_to_clients(&clients, message)?;
    }

    // remove client from list
    remove_client(stream, clients);

    Ok(())
}

// takes in a tcpstream and removes it from client list
fn remove_client(stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) {
    {
        // start locking
        let mut clients = match clients.lock() {
            Ok(c) => c,
            Err(poisoned) => poisoned.into_inner(),
        };
        let index = clients
            .iter()
            .position(|x| (*x).peer_addr().unwrap() == stream.peer_addr().unwrap())
            .unwrap();
        clients.remove(index);
    }
    // end locking
}

pub fn send_to_clients(
    clients: &Arc<Mutex<Vec<TcpStream>>>,
    message: &str,
) -> Result<(), Box<dyn Error + 'static>> {
    Ok({
        let mut clients = match clients.lock() {
            Ok(c) => c,
            Err(poisoned) => poisoned.into_inner(),
        };

        for client in clients.iter_mut() {
            client.write_all(message.as_bytes())?;
        }
    })
}

pub fn server(socket: &SocketAddr) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&socket)?;
    let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new()));
    let client_names = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    println!("Started listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream?;

        {
            // start lock
            let mut c = clients.lock().unwrap();
            c.push(stream.try_clone()?);
        } // end lock

        let client_copy = Arc::clone(&clients);
        let name_copy = Arc::clone(&client_names);

        thread::spawn(|| {
            let _ = handle_connection(stream, client_copy, name_copy);
        });
    }
    Ok(())
}

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
            if let Err(e) = tx_net.send(msg) {
                eprintln!("error sending message to UI: {e}");
            }
        }

        // get client message from UI
        if let Ok(msg) = rx_ui.try_recv() {
            if let Err(_) = send_message(&mut stream, msg) {
                continue;
            }
        }
    }
}

// gets message from TCP, converts it to Message and returns it
pub fn get_message(stream: &mut TcpStream) -> Result<MessageType, Box<dyn Error>> {
    let mut buf = Vec::new();
    let mut reader = BufReader::new(stream);
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
