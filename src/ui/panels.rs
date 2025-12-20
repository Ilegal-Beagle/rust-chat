use std::{
    fs::read
};

use crate::{
    message::{Message, MessageType},
    ui::app::App,
};

use egui::{
    Layout,
    Align
};
use uuid::Uuid;

impl App {
    pub fn message_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("message_entry").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let text_resp = ui.add(egui::TextEdit::singleline(&mut self.text)
                    .desired_width(250.0)
                    .hint_text("Type Here")
                );
                let send_button_resp = ui.button("send");
                let image_button_resp= ui.button("add image");

                // image handling
                if image_button_resp.clicked() {
                    self.file_dialog.pick_file();
                }

                self.file_dialog.update(ctx);

                if let Some(path) = self.file_dialog.take_picked() {
                    self.image_bytes = read(path.to_str().unwrap()).expect("invalid file path");
                }

                // When enter is pressed in text box or send button is pressed
                if (text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || send_button_resp.clicked()
                {
                    let time = chrono::Local::now().format("%I:%M %p").to_string();
                    let message = MessageType::Message(Message {
                            user_name: self.user_name.clone(),
                            profile_picture: self.profile_picture.clone(),
                            message: self.text.clone(),
                            image: self.image_bytes.clone(),
                            timestamp: time,
                            uuid: Uuid::new_v4().to_string(),
                            uuid_profile_picture: Uuid::new_v4().to_string(),
                    });

                    if let Some(net) = &self.client {
                        net.send(message, &self.rt_handle);
                    }
                    
                    self.text.clear();
                    self.image_bytes.clear();
                }

            });
        });
    }

    pub fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("user_panel")
            .resizable(false)
            .exact_width(100.0)
            .show(ctx, |ui| {

                ui.vertical(|ui| {
                    ui.heading(egui::RichText::new("Users"));
                    ui.separator();

                    for (key, _) in &mut self.users {
                        ui.label(key);
                    }
                });

                ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                    let emoji_button = ui.button("emojis");
                    self.emoji_popup(&emoji_button, ui);

                    let gif_button = ui.button("GIFs");
                    self.gif_popup(&gif_button, ui);
                });

            });
    }

    pub fn chat_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chat Room");
            ui.separator();
            egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink(false)
            .show(ui, |ui| {
                for msg in self.messages.iter_mut() {
                    match msg {
                        MessageType::Message(msg) => {ui.add(msg);},
                        MessageType::Notification(msg) => {ui.add(msg);},
                        MessageType::Handshake(_) => {},
                        MessageType::UserList(msg) => {
                            self.users = msg.clone();
                        },
                        MessageType::Disconnect(_) => {},
                    }
                }
            });
        });
    }

}