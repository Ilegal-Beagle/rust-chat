use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, fmt::Debug, sync::Arc
};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Message(Message),
    Notification(Notification),
    Connect(Connect),
    UserList(HashMap<String, String>),
    Disconnect(Disconnect),
}

// MESSAGE
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub user_name: String,
    pub profile_picture: Vec<u8>,
    pub message: String,
    pub image: Arc<Vec<u8>>,
    pub timestamp: String,
    pub uuid: String,
    pub uuid_profile_picture: String,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            user_name: "default".to_owned(),
            profile_picture: Vec::<u8>::new(),
            message: "default message".to_owned(),
            image: Arc::new(Vec::<u8>::new()),
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
            self.profile_picture.clone(),
        )
        .fit_to_original_size(0.5);

        let bytes = Arc::new(&self.image).to_vec();
        let attachment =
            egui::Image::from_bytes(format!("bytes://{}", self.uuid), bytes)
                .max_size(egui::vec2(250.0, 250.0))
                .fit_to_exact_size(egui::vec2(250.0, 250.0));

        let response = ui
            .horizontal(|ui| {
                ui.add(profile_pic);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&self.user_name).strong().italics());
                        ui.label(egui::RichText::new(&self.timestamp).weak().italics());
                    });
                    ui.label(egui::RichText::new(&self.message));
                });
            })
            .response;
        if !self.image.is_empty() {
            ui.add(attachment);
        }

        response
    }
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connect {
    pub user_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Disconnect {
    pub user_name: String,
    pub ip: String,
}
