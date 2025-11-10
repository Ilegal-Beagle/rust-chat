// ui.rs
use std::thread;

pub struct Chat {
    text: String,
    init: bool,
    messages: Vec<Message>,
}

struct Message {
    user_name: String,
    message: String,
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            text: "".to_owned(),
            init: true,
            messages: Vec::new(),
        }
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

impl eframe::App for Chat {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.init {
            thread::spawn(|| {
                // server();
            });
            self.init = false;
        }

        egui::SidePanel::right("user_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("username");
            });
        });

        egui::TopBottomPanel::bottom("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let response = ui.text_edit_singleline(&mut self.text);

                // When enter is pressed in text box or send button is pressed
                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) ||
                    ui.button("send").clicked(){
                    
                    let msg = Message {
                        user_name: "default".to_string(),
                        message: self.text.clone(),
                    };
                    self.messages.push(msg);

                    self.text.clear();
                }
            });
        });

       egui::CentralPanel::default().show(ctx, |ui| {
           for msg in self.messages.iter_mut() {
                ui.add(msg);
           }
       });

    }
}
