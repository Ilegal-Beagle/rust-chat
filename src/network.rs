// network.rs
use std::{
    str,
    io::{prelude::*, Result},
    net::{TcpStream, SocketAddr}, //, IpAddr, Ipv4Addr},
    time::{Duration},
};

use crate::ui::chat::Message;

pub fn try_connect(address: &SocketAddr, timeout: Duration) -> bool {
    match TcpStream::connect_timeout(address, timeout) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn handle_connection(mut stream: TcpStream) -> Result<()>{
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

                // send to serve
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
