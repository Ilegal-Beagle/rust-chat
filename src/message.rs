use std::{
    collections::HashMap,
    fmt::{self, Debug},
    net::SocketAddr,
    sync::Arc
};
use tokio::{
    net::TcpStream,
    io::WriteHalf,
    sync::Mutex,
};
use serde::{Deserialize, Serialize};
use crate::network::{helpers};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Message(Message),
    Notification(Notification),
    Handshake(Handshake),
    UserList(HashMap<String, String>),
    Disconnect(Disconnect),
}

impl MessageType {
    pub fn handle(
        self,
        user_list: &mut HashMap<String, String>,
        clients: &Arc<Mutex<HashMap<SocketAddr, WriteHalf<TcpStream>>>>,
    ) {
        match self {
            MessageType::Message(_) => {},
            MessageType::Notification(_) => {}
            MessageType::Handshake(handshake) => {
                match user_list.contains_key(&handshake.user_name) {
                    true => {
                        user_list.remove(&handshake.user_name);
                    }
                    false => {
                        user_list.insert(handshake.user_name, "".to_string());
                    }
                };

                // create user list message
                let list = MessageType::UserList(user_list.clone());

                let mut clients_clone = Arc::clone(clients);
                tokio::spawn(async move {
                    match helpers::send_to_clients(&mut clients_clone, list).await {
                        Ok(_) => {},
                        Err(e) => eprintln!("Error sending to clients: {}", e),
                    };
                });
            }
            MessageType::UserList(_) => {},
            MessageType::Disconnect(disconnect) => {
                match user_list.contains_key(&disconnect.user_name) {
                    true => {
                        user_list.remove(&disconnect.user_name);
                    }
                    false => {}
                };

                // create user list message
                let list = MessageType::UserList(user_list.clone());

                let mut clients_clone = Arc::clone(clients);
                tokio::spawn(async move {
                    match helpers::send_to_clients(&mut clients_clone, list).await {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Error sending to clients: {}", e);
                        },
                    };
                });

            },
        }
    }
}

// MESSAGE
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub user_name: String,
    pub profile_picture: Vec<u8>,
    pub message: String,
    pub image: Vec<u8>,
    pub timestamp: String,
    pub uuid: String,
    pub uuid_profile_picture: String,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User: {}\nMessage: {}\nHas Image: {:?}", self.user_name, self.message, self.image)
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            user_name: "default".to_owned(),
            profile_picture: Vec::<u8>::new(),
            message: "default message".to_owned(),
            image: Vec::<u8>::new(),
            timestamp: chrono::Local::now().to_string(),
            uuid: Uuid::new_v4().to_string(),
            uuid_profile_picture: Uuid::new_v4().to_string(),
        }
    }
}

impl egui::Widget for &mut Message {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let profile_pic = egui::Image::from_bytes(
            format!("bytes://{}", self.uuid_profile_picture),
            self.profile_picture.clone()).fit_to_original_size(0.5);

        let attachment = egui::Image::from_bytes(
            format!("bytes://{}", self.uuid),
            self.image.clone())
                .max_size(egui::vec2(250.0, 250.0))
                .fit_to_exact_size(egui::vec2(250.0, 250.0));
            

        let response = ui.horizontal(|ui|{
            ui.add(profile_pic);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&self.user_name).strong().italics());
                    ui.label(egui::RichText::new(&self.timestamp).weak().italics());
                });
                ui.label(
                    egui::RichText::new(&self.message)
                );
            });
        })
            .response;
                if !self.image.is_empty() {
                    ui.add(attachment);
                }

        response
    }
}

// NOTIFICATION
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notification {
    pub message: String,
}

impl egui::Widget for &mut Notification {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui
            .vertical(|ui| {
                ui.label(egui::RichText::new(&self.message).weak());
            })
            .response;
        response
    }
}

// HANDSHAKE
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Handshake {
    pub user_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
// DISCONNECT
pub struct Disconnect {
    pub user_name: String,
    pub ip: String,
}