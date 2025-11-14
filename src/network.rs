// network.rs
use std::{
    str,
    thread,
    io::{prelude::*},
    net::{TcpListener, TcpStream, SocketAddr}, //, IpAddr, Ipv4Addr},
    time::{Duration},
    sync::mpsc::{Sender, Receiver},
    error::{Error},
};

use crate::ui::chat::Message;

pub fn try_connect(address: &SocketAddr, timeout: Duration) -> bool {
    match TcpStream::connect_timeout(address, timeout) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn handle_connection(mut stream: TcpStream) -> std::io::Result<()>{
    let mut buf = [0u8; 128]; // where the message recved is stored
    println!("client connected");

    loop {
        match stream.read(&mut buf) {
            Ok(0) => { break; },
            
            Ok(bytes_read) => {
                // UTF8 to str
                let message = match str::from_utf8(&buf[..bytes_read]) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Invalid UTF8: {e}");
                        continue;
                    },
                };

                // str to Message
                let deserialized_msg: Message = match serde_json::from_str(&message) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Could not deserialize message: {e}");
                        continue;
                    },
                };

                println!("{}", deserialized_msg);

                // send
                match stream.write_all(message.as_bytes()) {
                    Ok(_) => {println!("message sent to client");},
                    Err(e) => {
                        eprintln!("Could not write to client: {e}");
                        continue;
                    },
                }
            },

            Err(e) => {
                eprintln!("Could not read file stream: {e}");
                continue;
            }
        }
    }

    println!("client disconnected");
    Ok(())
}

pub fn server(socket: &SocketAddr) {
    let listener = TcpListener::bind(&socket).expect("couldnt bind");
    println!("Started listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            let _ = handle_connection(stream);
        });
    }
}

// does the client functions. gets a message from user and sends and reveives it back from server.
pub fn client(socket: &SocketAddr, rx_ui: &Receiver<Message>, tx_net: &Sender<Message>) {
    let mut stream = TcpStream::connect(&socket).expect("could not connect");
    stream.set_read_timeout(Some(Duration::new(0, 10000))).expect("could not set timeout");
    println!("connected to server.");

    loop {
        // get client message from UI
        match rx_ui.try_recv() {
            
            Ok(msg) => {
                // serialize message
                let serialized_msg = match serde_json::to_string(&msg) {
                    Ok(msg) => msg,                                
                    Err(e) => {
                        eprintln!("error in serializing message: {e:?}");
                        continue;
                    },
                };
                
                // send message to server
                match stream.write_all(serialized_msg.as_bytes()) {
                    Ok(_) => {println!("message sent to server")},
                    Err(e) => {println!("Error in sending message: {e:?}")},
                };

                // receive repsonse from server
                let msg = match get_message(&mut stream) {
                    Ok(msg) => msg,
                    Err(e) => {
                        println!("Error: {e}");
                        continue;
                    },
                };

                match tx_net.send(msg) {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("Error sending message: {e}");
                        continue;
                    }
                }
            },

            Err(_) => {/* Do nothing if nothing was recved */},
        }
        
    }
}

// gets message from TCP, converts it to Message and returns it
pub fn get_message(stream: &mut TcpStream) -> Result<Message, Box<dyn Error>> {
    let mut buf = [0u8; 128];
    let bytes_read = stream.read(&mut buf)?;
    
    // UTF8 to str
    let message = str::from_utf8(&buf[..bytes_read])?;

    // str to Message
    let deserialized_msg: Message = serde_json::from_str(&message)?;

    println!("{}", deserialized_msg);

    Ok(deserialized_msg)
}