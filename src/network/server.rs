use std::{
    collections::HashMap,
    error::Error,
    io::{self, prelude::*},
    net::{SocketAddr},
    sync::{Arc, Mutex},
    thread,
};
use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}};
use tokio::io::{AsyncReadExt, AsyncBufReadExt, BufReader};
use crate::{message::MessageType, network::{helpers::send_to_clients, server}};

// pub fn handle_connection(
//     stream: TcpStream,
//     clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
//     names: Arc<Mutex<HashMap<String, String>>>,
// ) -> Result<(), Box<dyn Error>> {

//     let mut buf = Vec::new();
//     let mut reader = BufReader::new(stream.try_clone()?);

//     loop {
//         match reader.read_until(b'\n', &mut buf) {
//             Ok(0) => break,

//             Ok(_) => {
//                 let message = std::str::from_utf8(&buf)?;
//                 let deserialized_msg: MessageType = serde_json::from_str(message)?;

//                 {
//                     let mut user_list = names.lock().unwrap_or_else(|p| p.into_inner());
//                     deserialized_msg.handle(&mut user_list, &clients);
//                 }

//                 send_to_clients(&clients, message)?;
//                 buf.clear();
//             }

//             Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,

//             Err(ref e)
//                 if e.kind() == io::ErrorKind::ConnectionReset
//                 || e.kind() == io::ErrorKind::ConnectionAborted => break,

//             Err(e) => return Err(Box::new(e)),
//         }
//     }


//     remove_client(stream, clients);

//     Ok(())
// }


// // takes in a tcpstream and removes it from client list
// fn remove_client(stream: TcpStream, clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>) {
//     let peer = match stream.peer_addr() {
//         Ok(p) => p,
//         Err(_) => return, // can't identify client
//     };

//     let mut clients = match clients.lock() {
//         Ok(c) => c,
//         Err(poisoned) => poisoned.into_inner(),
//     };

//     clients.remove(&peer);
// }

// pub fn server(socket: &SocketAddr) -> Result<(), Box<dyn Error>> {
//     let listener = TcpListener::bind(&socket).await?;
//     // let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new()));
//     let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, TcpStream>::new()));
//     let client_names = Arc::new(Mutex::new(HashMap::<String, String>::new()));
//     println!("Started listening on port 7878");

//     for stream in listener.incoming() {
//         let stream = stream?;

//         { // start lock
//             let mut c = clients.lock().unwrap();
//             let addr = stream.peer_addr()?;
//             c.insert(addr, stream.try_clone()?);
//         } // end lock

//         let client_copy = Arc::clone(&clients);
//         let name_copy = Arc::clone(&client_names);

//         thread::spawn(|| {
//             let _ = handle_connection(stream, client_copy, name_copy);
//         });
//     }
//     Ok(())
// }

pub async fn server(socket: &SocketAddr) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(socket).await.unwrap();

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = [0; 128];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Connection closed by client: {}", addr);
                        return;
                    }
                    Ok(n) => n,

                    Err(e) => {
                        eprintln!("Failed to read from socket {}: {}", addr, e);
                        return;
                    }
                };

                if let Ok(s) = String::from_utf8(buf[..n].to_vec()) {
                    print!("Received from {}: {}", addr, s);
                } else {
                    println!("Received {} bytes of non-UTF8 data from {}", n, addr);
                }


                if let Err(e) = socket.write_all(&buf[..n]).await {
                    eprintln!("Failed to write to socket {}: {}", addr, e);
                    return;
                }
            }
        });
    }
}