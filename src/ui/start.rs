// ui.rs

pub struct Start {
    text: String,
}

impl Default for Start {
    fn default() -> Self {
        Self {
            text: "Enter Username Here".to_string(),
        }
    }
}

impl eframe::App for Start {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
       egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("hello there");
            let response = ui.text_edit_singleline(&mut self.text);

            if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) ||
                ui.button("Enter").clicked() {
                
                let username = self.text.to_string();
                println!("{}", username);
            }
       });

    }
}
