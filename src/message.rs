use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    fmt,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::network;

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Message(Message),
    Notification(Notification),
    Handshake(Handshake),
    UserList(HashMap<String, String>),
}

impl MessageType {
    pub fn handle(
        self,
        user_list: &mut HashMap<String, String>,
        clients: &Arc<Mutex<Vec<TcpStream>>>,
    ) {
        match self {
            MessageType::Message(_) => {}
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
                let mut serialized_msg = serde_json::to_string(&list).unwrap();
                serialized_msg.push_str("\n");

                // send to all clients
                network::send_to_clients(clients, &serialized_msg).unwrap();

                // send to clients
            }
            MessageType::UserList(_) => {}
        }
    }
}

// MESSAGE
#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub user_name: String,
    pub message: String,
    pub image: Vec<u8>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User: {}\nMessage: {}", self.user_name, self.message)
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            user_name: "default".to_owned(),
            message: "default message".to_owned(),
            image: Vec::<u8>::new(),
        }
    }
}

impl egui::Widget for &mut Message {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui
            .vertical(|ui| {
                ui.label(egui::RichText::new(&self.user_name).weak().italics());
                ui.label(&self.message);
            })
            .response;
        if !self.image.is_empty() {
            ui.add(
                egui::Image::from_bytes("bytes://image", self.image.clone())
                    .max_size(egui::vec2(250.0, 250.0))
                    .fit_to_exact_size(egui::vec2(250.0, 250.0)),
            );
        }

        response
    }
}

// NOTIFICATION
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
pub struct Handshake {
    pub user_name: String,
}
