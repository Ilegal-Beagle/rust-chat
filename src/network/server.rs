use std::{
    collections::HashMap,
    error::Error,
    net::{SocketAddr},
    sync::{Arc},
};
use tokio::{
    io::{
        AsyncBufReadExt,
        BufReader,
        WriteHalf,
        ReadHalf,
    }, 
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use crate::{message::MessageType, network::helpers::send_to_clients};

pub async fn server(socket: &SocketAddr) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(socket).await.unwrap();
    let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, WriteHalf<TcpStream>>::new()));
    let user_names = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    println!("now accepting clients");

    loop {
        let (stream, socket) = listener.accept().await.unwrap();
        let (reader, writer) = tokio::io::split(stream);

        {
            let mut clients = clients.lock().await;
            clients.insert(socket, writer);
        }

        let clients_clone = Arc::clone(&clients);
        let name_clone = Arc::clone(&user_names);

        tokio::spawn(async move {
            println!("client {:?} now being SERVED", socket);
            handle_connection(reader, clients_clone, name_clone).await;
        });
    }
}

async fn handle_connection(
    reader: ReadHalf<TcpStream>,
    clients: Arc<Mutex<HashMap::<SocketAddr, WriteHalf<TcpStream>>>>,
    names: Arc<Mutex<HashMap<String, String>>>) {
    let mut reader = BufReader::new(reader);
    let mut buf: Vec<u8> = Vec::new();

    loop {
        tokio::select! {
            res = reader.read_until(b'\n', &mut buf) => {
                match res {
                    Ok(0) => break,
                    Ok(_) => {
                        let msg = std::str::from_utf8(&buf).unwrap();
                        let deserialized_msg: MessageType = serde_json::from_str(&msg).unwrap();
                        let msg_cpy = deserialized_msg.clone();
        
                        {
                            let mut user_list = names.lock().await;
                            deserialized_msg.handle(&mut user_list, &clients);
                        }
        
                        let mut clients_clone = Arc::clone(&clients);
                        match send_to_clients(&mut clients_clone, msg_cpy).await {
                            Ok(_) => {},
                            Err(e) => {
                                eprintln!("Error sending to multiple clients: {}", e);
                            },
                        };
                        buf.clear();
                    },

                    Err(e) => {
                        eprintln!("Error in reading from client: {}", e);
                        continue;
                    },
                };
            }
        }
    }
}
