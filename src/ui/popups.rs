use crate::App;
use egui::vec2;

impl App {
    #[allow(unused_variables)]
    pub fn emoji_popup(&mut self, resp: &egui::Response, ui: &mut egui::Ui) {
        egui::Popup::menu(&resp)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .width(50.0)
            .show(|ui| {

                ui.heading("emojis");
                
                egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                
                    egui::Grid::new("emoji_board")
                        .spacing(vec2(1.0, 1.0))
                        .show(ui, |ui| {
                            for i in 1..75 {
                                
                                if i % 3 == 1 {
                                    ui.end_row();
                                }
                                
                                let emoji = char::from_u32(0x1F600+i).unwrap();
                                let button_text = egui::RichText::new(emoji.to_string())
                                    .size(30.0);
                                if ui.button(button_text).clicked() {
                                    self.text.push(emoji);
                                }
                            }
                    });
                });
        });
    }

    #[allow(unused_variables)]
    pub fn gif_popup(&mut self, resp: &egui::Response, ui: &mut egui::Ui) {
        
        // when the button is clicked
        if resp.clicked() {
            let mut api_clone = self.tenor_api.clone();

            self.rt_handle.spawn(async move {
                
                match api_clone.featured(1).await {
                    
                    Ok(resp) => {
                        println!("{:?}", resp);
                        // send to ui
                    },

                    Err(e) => {
                        eprintln!("{}", e);
                    },
                };
            });
        }

        egui::Popup::menu(&resp)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.heading("gifs");
            });
    }

}