// ui.rs
use std::thread;

pub struct MyApp {
    name: String,
    text: String,
    age: u32,
    init: bool,
}

struct Message {
    user_name: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            text: "".to_owned(),
            age: 42,
            init: true,
        }
    }
}

impl Default for Message {

    fn default() -> Self {
        Self {
            user_name: "default".to_owned(),
        }
    }
}

impl egui::Widget for &mut Message {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.vertical(|ui| {
            ui.label("username");
            ui.label("message");
        }).response;

        response
    }
}

impl eframe::App for MyApp {
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
                ui.text_edit_singleline(&mut self.text);
                ui.button("send");
            });
        });

       egui::CentralPanel::default().show(ctx, |ui| {
           ui.add(&mut Message::default());
       });
    }
}
