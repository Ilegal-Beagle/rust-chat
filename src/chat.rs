// ui.rs
use crate::network;

use std::{
    thread,
    fmt,
    net::{SocketAddr, IpAddr, Ipv4Addr},
    time::{Duration},
    sync::{mpsc},
};

use serde::{Serialize, Deserialize};
use egui::RichText;

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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    user_name: String,
    message: String,
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

impl App {
    pub fn new() -> Self {
        const SOCKET: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7878);
        const TIMEOUT: Duration = Duration::new(5, 0);

        let (tx_ui, rx_ui) = mpsc::channel::<Message>();
        let (tx_net, rx_net) = mpsc::channel::<Message>();
        
        // if no server is found
        if !network::try_connect(&SOCKET, TIMEOUT) {
            thread::spawn( move || {
                let _ = network::server(&SOCKET);
            });
        }
            thread::sleep(Duration::new(2, 0));
            thread::spawn(move || {
                let _ = network::client(&SOCKET, &rx_ui, &tx_net);
            } );

        Self {
            user_name: "Default".to_string(),
            text: "".to_owned(),
            messages: Vec::new(),
            users: Vec::new(),
            tx: tx_ui,
            rx: rx_net,
            current_state: State::Start,
        }
    }

    fn render_chat(&mut self, ctx: &egui::Context) {
        match self.rx.try_recv() {
            Ok(msg) => self.messages.push(msg),
            Err(_) => {},
        }
        
        egui::SidePanel::right("user_panel").show(ctx, |ui| {
            ui.vertical( |ui| {
                ui.label(egui::RichText::new("Users\n").weak());
                for user in self.users.iter_mut() {
                    ui.label(user.to_string());
                }
            });
        });
        
        egui::TopBottomPanel::bottom("message_entry").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let response = ui.text_edit_singleline(&mut self.text);
        
                // When enter is pressed in text box or send button is pressed
                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) ||
                    ui.button("send").clicked() {
                    
                    self.tx.send( Message {
                        user_name: self.user_name.clone(),
                        message: self.text.clone(),
                    } ).unwrap();
        
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

    fn render_start(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading(RichText::new("Rust Chat").heading().strong());
                
                ui.horizontal(|ui| {
                    ui.label("Username: ");
                    ui.text_edit_singleline(&mut self.user_name);
                });

                if ui.button("Enter").clicked()  {
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