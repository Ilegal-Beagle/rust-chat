// network.rs
use std::{
    str,
    thread,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream, SocketAddr},
    time::{Duration},
    sync::{Arc, Mutex},
    sync::mpsc::{Sender, Receiver},
    error::{Error},
};

use crate::message::Message;

pub fn try_connect(address: &SocketAddr, timeout: Duration) -> bool {
    match TcpStream::connect_timeout(address, timeout) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn handle_connection(mut stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 128];

    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 { break; }
        let message = str::from_utf8(&buf[..bytes_read])?;

        // send received message to all clients
        { // start locking
            let mut clients = match clients.lock() {
                Ok(c) => c,
                Err(poisoned) => poisoned.into_inner(),
            };

            for client in clients.iter_mut() {
                client.write_all(message.as_bytes())?;
            }
        } // end locking
    }

    // remove client from list
    { // start locking
        let mut clients = match clients.lock() {
            Ok(c) => c,
            Err(poisoned) => poisoned.into_inner(),
        };
        let index = clients.iter().position(|x| (*x).peer_addr().unwrap() == stream.peer_addr().unwrap()).unwrap();
        clients.remove(index);
    } // end locking

    Ok(())
}

//this function does the thing that u need for the thing to that thing for that thang!
pub fn server(socket: &SocketAddr) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&socket)?;
    let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new())); // creates vec of clients w/ locking and ref-counting
    println!("Started listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream?;
        
        { // start lock
            let mut c = clients.lock().unwrap();
            c.push(stream.try_clone()?);
        } // end lock

        let client_copy = Arc::clone(&clients);

        thread::spawn(|| {
            let _ = handle_connection(stream, client_copy);
        });

    }
    Ok(())
}

// does the client functions. gets a message from user and sends and reveives it back from server.
pub fn client(socket: &SocketAddr, rx_ui: Receiver<Message>, tx_net: Sender<Message>) -> Result<(), Box<dyn Error>>{
    let mut stream = TcpStream::connect(&socket)?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    println!("connected to server.");

    loop {

        // receive repsonse from server and send it to UI
        match get_message(&mut stream) {
            Ok(msg) => {
                match tx_net.send(msg) {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("Error sending message: {e}");
                        continue;
                    }
                }
            },
            Err(_) => {},
        };

        // get client message from UI
        match rx_ui.try_recv() {
            
            Ok(msg) => {
                // serialize message
                let mut serialized_msg = match serde_json::to_string(&msg) {
                    Ok(msg) => msg,                                
                    Err(e) => {
                        eprintln!("error in serializing message: {e:?}");
                        continue;
                    },
                };

                serialized_msg.push_str("\n");
                
                // send message to server
                stream.write_all(serialized_msg.as_bytes())?;

            },

            Err(_) => {/* Do nothing if nothing was recved */},
        }
        
    }
}

// gets message from TCP, converts it to Message and returns it
pub fn get_message(stream: &mut TcpStream) -> Result<Message, Box<dyn Error>> {
    let mut buf = Vec::new();
    let mut reader = BufReader::new(stream);
    let bytes_read = reader.read_until(b'\n', &mut buf)?;
    let message = str::from_utf8(&buf)?;

    let deserialized_msg: Message = serde_json::from_str(&message)?;
    Ok(deserialized_msg)
}