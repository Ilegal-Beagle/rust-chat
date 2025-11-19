// ui.rs
use crate::network;

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::mpsc,
    thread,
    time::Duration
};

use egui::{RichText};
use egui_file_dialog::FileDialog;
use serde::{Deserialize, Serialize};

enum State {
    Start,
    Chat,
}

pub struct App {
    user_name: String,
    text: String,
    messages: Vec<Message>,
    users: Vec<String>,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    current_state: State,
    file_dialog: FileDialog,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    user_name: String,
    message: String,
    image_path: String,
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
            image_path: "".to_string(),
        }
    }
}

impl egui::Widget for &mut Message {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui
            .vertical(|ui| {
                ui.label(egui::RichText::new(&self.user_name).weak().italics());
                ui.label(&self.message);
                if self.image_path != "".to_string() {
                    ui.image(&self.image_path);
                }
            })
            .response;

        response
    }
}

impl App {
    pub fn new() -> Self {
        const SOCKET: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7878);
        const TIMEOUT: Duration = Duration::new(5, 0);

        let (tx_ui, rx_ui) = mpsc::channel::<Message>();
        let (tx_net, rx_net) = mpsc::channel::<Message>();

        // if no server is found
        if !network::try_connect(&SOCKET, TIMEOUT) {
            thread::spawn(move || {
                let _ = network::server(&SOCKET);
            });
        }
        thread::sleep(Duration::new(2, 0));
        thread::spawn(move || {
            let _ = network::client(&SOCKET, &rx_ui, &tx_net);
        });

        Self {
            user_name: "Default".to_string(),
            text: "".to_owned(),
            messages: Vec::new(),
            users: Vec::new(),
            tx: tx_ui,
            rx: rx_net,
            current_state: State::Start,
            file_dialog: FileDialog::new(),
        }
    }

    fn render_chat(&mut self, ctx: &egui::Context) {
        
        match self.rx.try_recv() {
            Ok(msg) => self.messages.push(msg),
            Err(_) => {}
        }

        egui::TopBottomPanel::bottom("message_entry").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // ui.set_width(250.0);
                let text_resp = ui.text_edit_singleline(&mut self.text);
                let send_button_resp = ui.button("send");
                let image_button_resp= ui.button("add image");

                // When enter is pressed in text box or send button is pressed
                if (text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || send_button_resp.clicked()
                {
                    self.tx
                        .send(Message {
                            user_name: self.user_name.clone(),
                            message: self.text.clone(),
                            image_path: "".to_string(),
                        })
                        .unwrap();

                    self.text.clear();
                }


                // image handling
                if image_button_resp.clicked() {
                    self.file_dialog.pick_file();
                }

                self.file_dialog.update(ctx);

                if let Some(path) = self.file_dialog.take_picked() {
                    let mut p = path.to_str().expect("eeh").to_string();
                    p.insert_str(0, "file://");
                    println!("{:?}", path);

                    // self.tx
                    // .send(Message {
                    //     user_name: self.user_name.clone(),
                    //     message: self.text.clone(),
                    //     image_path: path.to_str().expect("eeh").to_string(),
                    // })
                    // .unwrap();

                    self.messages.push(Message {
                        user_name: self.user_name.clone(),
                        message: self.text.clone(),
                        image_path: p,
                    });

                }

            });
        });


        egui::SidePanel::right("user_panel")
            .resizable(false)
            .exact_width(100.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Users\n").weak());
                    for user in self.users.iter_mut() {
                        ui.label(user.to_string());
                    }
                });
            });


        egui::CentralPanel::default().show(ctx, |ui| {
            for msg in self.messages.iter_mut() {
                ui.add(msg);
            }
        });
    }

    fn render_start(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading(RichText::new("Rust Chat").heading().strong());

                ui.horizontal(|ui| {
                    ui.label("Username: ");
                    ui.text_edit_singleline(&mut self.user_name);
                });

                if ui.button("Enter").clicked() {
                    self.current_state = State::Chat;
                }
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.current_state {
            State::Start => self.render_start(ctx),
            State::Chat => self.render_chat(ctx),
        }
    }
}
