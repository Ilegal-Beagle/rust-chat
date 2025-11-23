use std::{
    collections::HashMap,
    error::Error,
    io::{self, BufReader, prelude::*},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex,},
    thread,
};

use crate::{message::MessageType, network::helpers::send_to_clients};

pub fn handle_connection(
    stream: TcpStream,
    clients: Arc<Mutex<Vec<TcpStream>>>,
    names: Arc<Mutex<HashMap<String, String>>>,
) -> Result<(), Box<dyn Error>> {

    let mut buf = Vec::new();
    let mut reader = BufReader::new(stream.try_clone()?);

    loop {
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) => break,

            Ok(_) => {
                let message = std::str::from_utf8(&buf)?;
                let deserialized_msg: MessageType = serde_json::from_str(message)?;

                {
                    let mut user_list = names.lock().unwrap_or_else(|p| p.into_inner());
                    deserialized_msg.handle(&mut user_list, &clients);
                }

                send_to_clients(&clients, message)?;
                buf.clear();
            }

            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,

            Err(ref e)
                if e.kind() == io::ErrorKind::ConnectionReset
                || e.kind() == io::ErrorKind::ConnectionAborted => break,

            Err(e) => return Err(Box::new(e)),
        }
    }

    remove_client(stream, clients);

    Ok(())
}


// takes in a tcpstream and removes it from client list
fn remove_client(stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) {
    let peer = match stream.peer_addr() {
        Ok(p) => p,
        Err(_) => return, // can't identify client
    };

    let mut clients = match clients.lock() {
        Ok(c) => c,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Some(pos) = clients
        .iter()
        .position(|c| c.peer_addr().map(|addr| addr == peer).unwrap_or(false))
    {
        clients.remove(pos);
    }
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