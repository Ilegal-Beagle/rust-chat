use std::error::Error;

use crate::App;
use egui::vec2;
use crate::gif;

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
                                    self.io.message_text.push(emoji);
                                }
                            }
                    });
                });
        });
    }

    #[allow(unused_variables)]
    pub fn gif_popup(&mut self, resp: &egui::Response, ui: &mut egui::Ui) {
        
        // load featured gifs by default
        if resp.clicked() {
            self.fetch_featured_gifs();
        }

        // get response from api
        match self.network.rx.try_recv() {
            Ok(resp) => {
                for result in resp {
                    let k = result.id.clone();
                    let v = result.clone();
                    self.gif_cache.insert(k, v);
                }
            },
            Err(_) => {
            }
        }

        // supposed to dynamically change size of popup menu
        let height = self.env.window_size.x / 2.0;
        let width = self.env.window_size.x / 2.0;

        // render the popup
        egui::Popup::menu(&resp)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.heading("gifs");
                egui::ScrollArea::vertical()
                    .max_width(width)
                    .max_height(height)
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.label("Search: ");
                        ui.text_edit_singleline(&mut self.io.gif_search_text);

                        if ui.button("search").clicked() {
                            self.gif_cache.clear();
                            self.fetch_search_gifs();
                        }
                        
                        ui.separator();

                        // show loaded gifs
                        for (id, gif) in &self.gif_cache {
                            let image = egui::Image::new(gif.tinygif_url.clone())
                                .fit_to_original_size(0.50)
                                .corner_radius(5);
                            let button = egui::Button::image(image);
                            let button_resp = ui.add(button);                            
                            
                            // if button clicked, send the image
                            if button_resp.clicked() {
                                self.io.image_bytes = gif.gif_bytes.clone();
                            }
                        }
                    });
            });
    }

    fn fetch_featured_gifs(&mut self) {
        let mut api_clone = self.tenor_api.clone();
        let tx_clone = self.network.tx.clone();
    
        self.rt_handle.spawn(async move {
            let mut gif_results = Vec::<gif::Gif>::new();

            // get api response
            let resp = match api_clone.featured(2).await {                    
                Ok(resp) => {resp},
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                },
            };
        
            // create a vec of gif structs to send back
            for i in resp {
                let bytes = reqwest::get(i.url.clone())
                    .await
                    .unwrap()
                    .bytes()
                    .await
                    .unwrap()
                    .to_vec();
        
                gif_results.push(
                    gif::Gif {
                        id: i.id.clone(),
                        url: i.url.clone(),
                        tinygif_url: i.tinygif_url.clone(),
                        gif_bytes: bytes,
                    }
                );
            }
    
            let _ = tx_clone.try_send(gif_results);
            
        });
    }

    fn fetch_search_gifs(&mut self) {
        let mut api_clone = self.tenor_api.clone();
        let tx_clone = self.network.tx.clone();
        let search = self.io.gif_search_text.clone();
    
        self.rt_handle.spawn(async move {
    
            // get api response
            let resp = match api_clone.search(search, 2).await {                    
                Ok(resp) => {resp},
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                },
            };
    
            let mut gif_results = Vec::<gif::Gif>::new();
    
            // create a vec of gif structs to send back
            for i in resp {
                let bytes = reqwest::get(i.url.clone())
                    .await
                    .unwrap()
                    .bytes()
                    .await
                    .unwrap()
                    .to_vec();
        
                gif_results.push(
                    gif::Gif {
                        id: i.id.clone(),
                        url: i.url.clone(),
                        tinygif_url: i.tinygif_url.clone(),
                        gif_bytes: bytes,
                    }
                );
            }
    
            let _ = tx_clone.try_send(gif_results);
    
        });
    }

}