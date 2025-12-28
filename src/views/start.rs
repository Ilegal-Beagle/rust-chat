use std::fs::read;
use crate::App::{self};
use crate::views::state::View;
use egui::{RichText, vec2};

impl App {
        // rendering the start state UI
    pub fn render_start(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading(RichText::new("Rust Chat").heading().strong());
                ui.separator();

                ui.label("Username: ");
                ui.add(egui::TextEdit::singleline(&mut self.user.local.name).desired_width(100.0));
                ui.add_space(5.0);

                ui.label("ip and port: ");
                ui.add(egui::TextEdit::singleline(&mut self.network.ip_str).desired_width(100.0));
                ui.add_space(10.0);

                ui.heading(RichText::new("Choose a Profile Picture"));
                egui::Grid::new("profile_pictures").show(ui, |ui| {
                    for path in self.user.profile_picture_list.iter() {
                        let image = egui::Image::from_uri(path);
                        if ui.add(
                            egui::Button::image(image.fit_to_fraction(vec2(2.0, 2.0)))
                        ).clicked() {
                            let p = path.trim_start_matches("file://");
                            self.user.local.picture = read(p).unwrap(); // should never fail
                        }
                    }
                });

                ui.add_space(10.0);
                
                if ui.button("Enter").clicked()  {
                    match self.network.ip_str.as_str().parse() {
                        Ok(ip) => {
                            self.network.socket_addr = ip;
                            self.view = View::Connect;
                        },
                        Err(_) => {
                            self.network.bad_ip_msg = true;
                        },
                    }
                }

                if self.network.bad_ip_msg {
                    ui.colored_label(egui::Color32::DARK_RED, "Invalid IP chosen");
                }
            });
        });
    }

}