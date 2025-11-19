use std::fmt;
use serde::{Serialize, Deserialize};

pub enum MessageType {
    Message,
    Notification,
    Handshake,
}


// MESSAGE
#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub user_name: String,
    pub message: String,
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
        }
    }
}

impl egui::Widget for &mut Message {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.vertical(|ui| {
            ui.label(egui::RichText::new(&self.user_name).weak().italics());
            ui.label(&self.message);
        }).response;

        response
    }
}

// // NOTIFICATION
// #[derive(Serialize, Deserialize, Debug)]
// pub struct Notification {
//     pub message: String,
// }

// impl egui::Widget for &mut Message {
//     fn ui(self, ui: &mut egui::Ui) -> egui::Response {
//         let response = ui.vertical(|ui| {
//             ui.label(egui::RichText::new(&self.message).weak());
//         });
//     }
// }

// // HANDSHAKE
// #[derive(Serialize, Deserialize, Debug)]
// pub struct Handshake {
//     pub user_name: String,
// }
