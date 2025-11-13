// network.rs
use std::{
    str,
    io::{prelude::*, stdin},
    net::{TcpListener, TcpStream, SocketAddr}, //, IpAddr, Ipv4Addr},
    time::{Duration},
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
    let mut bytes_read = 1;
    println!("client connected");

    loop {
        match stream.read(&mut buf) {
            Ok(0) => { break; },
            
            Ok(bytes_read) => {
                let message = match str::from_utf8(&buf[..bytes_read]) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Invalid UTF8: {e}");
                        continue;
                    },
                }

                let deserialized_msg: Message = match serde_json::from_str(&message) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Could not deserialize message: {e}");
                        continue;
                    },
                }

                // write to server
            },
        }
    }

    // while bytes_read != 0 {
    //     bytes_read = stream.read(&mut buf)?;

    //     // bytes to string
    //     let message = str::from_utf8(&buf[..bytes_read]).expect("utf8 -> str failed");

    //     // string to object
    //     let deserialized: Message = serde_json::from_str(&message).expect("str -> Message failed");
    //     println!("Message received: {}", deserialized);

    //     stream.write_all(message.as_bytes())?;
    // }

    println!("client disconnected");
    Ok(())
}

pub fn get_input() -> String {
    let mut buffer:String = String::new();
    let _ = stdin().read_line(&mut buffer);
    return buffer.trim().to_string();
}

