use crate::{message::MessageType, room::Room};
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader, ReadHalf},
    net::{TcpListener, TcpStream},
};

pub async fn server(socket: &SocketAddr) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(socket).await.unwrap();
    let rooms: Arc<HashMap<&str, Arc<Room>>> = Arc::new(HashMap::from([
        ("main", Arc::new(Room::new("main"))),
        ("general", Arc::new(Room::new("general"))),
    ]));

    println!("now accepting clients");

    // Listen for clients
    loop {
        // get client
        let (stream, socket) = listener.accept().await.unwrap();
        let (reader, writer) = tokio::io::split(stream);

        // send the names of the rooms available to the client

        // add them to user list
        {
            let mut clients = rooms["main"].client_sockets.lock().await;
            clients.insert(socket, writer);
        }

        // handle the client
        let room_clone = Arc::clone(&rooms["main"]);
        tokio::spawn(async move {
            println!("client {:?} now being SERVED", socket);
            handle_connection(reader, room_clone).await;
        });
    }
}

async fn handle_connection(reader: ReadHalf<TcpStream>, room: Arc<Room>) {
    let mut reader = BufReader::new(reader);
    let mut buf: Vec<u8> = Vec::new();

    loop {
        tokio::select! {
            // get message from client
            res = reader.read_until(b'\n', &mut buf) => {
                match res {
                    // client ended connection, disconnect the client
                    Ok(0) => {
                        break;
                    },
                    // message recieved from a client
                    Ok(_) => {
                        let msg = std::str::from_utf8(&buf).unwrap();
                        let deserialized_msg: MessageType = match serde_json::from_str(&msg) {
                            Ok(msg) => {
                                msg
                            },
                            Err(e) => {
                                eprintln!("Error deserializing Message: {e}");
                                continue;
                            }
                        };

                        let msg_cpy = deserialized_msg.clone();

                        {
                            let mut user_list = room.client_names.lock().await;
                            match deserialized_msg {
                                MessageType::Connect(m) => {
                                    user_list.insert(m.user_name, "online".to_string());
                                },
                                MessageType::Disconnect(m) => {
                                    user_list.remove(&m.user_name);
                                },
                                _ => {}
                            }
                        }

                        room.broadcast(msg_cpy).await;

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
