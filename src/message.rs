use std::fmt;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Message(Message),
    Notification(Notification),
    Handshake(Handshake),
    UserList(HashMap<String, String>),
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
        let response = ui.vertical(|ui| {
            ui.label(egui::RichText::new(&self.user_name).weak().italics());
            ui.label(&self.message);
        }).response;
        if !self.image.is_empty() {
            ui.add(
                egui::Image::from_bytes("bytes://image", self.image.clone())
                .max_size(egui::vec2(250.0, 250.0))
                .fit_to_exact_size(egui::vec2(250.0, 250.0))
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
        let response = ui.vertical(|ui| {
            ui.label(egui::RichText::new(&self.message).weak());
        }).response;
        response
    }
}

// HANDSHAKE
#[derive(Serialize, Deserialize, Debug)]
pub struct Handshake {
    pub user_name: String,
}
