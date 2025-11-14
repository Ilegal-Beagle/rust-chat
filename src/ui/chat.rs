// ui.rs
use crate::network;

use std::{
    thread,
    fmt,
    io::{prelude::*},
    net::{SocketAddr, IpAddr, Ipv4Addr},
    time::{Duration},
    sync::{mpsc},
};

use serde::{Serialize, Deserialize};

pub struct App {
    user_name: String,
    text: String,
    messages: Vec<Message>,
    users: Vec<String>,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
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

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            user_name: "Default".to_string(),
            text: "".to_owned(),
            messages: Vec::new(),
            users: Vec::new(),
            tx: tx,
            rx: rx,
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

impl App {
    pub fn new() -> Self {
        const SOCKET: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7878);
        const TIMEOUT: Duration = Duration::new(5, 0);

        let (tx_ui, rx_ui) = mpsc::channel::<Message>();
        let (tx_net, rx_net) = mpsc::channel::<Message>();
        
        if !network::try_connect(&SOCKET, TIMEOUT) {

            // create a server
            thread::spawn( move || {
                network::server(&SOCKET);
            });
            

        } else {
            // create a client
            thread::spawn(move || {
                network::client(&SOCKET, &rx_ui, &tx_net);
            } );
        }

        Self {
            user_name: "Default".to_string(),
            text: "".to_owned(),
            messages: Vec::new(),
            users: Vec::new(),
            tx: tx_ui,
            rx: rx_net,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        match self.rx.try_recv() {
            Ok(msg) => self.messages.push(msg),
            Err(_) => {},
        }

        // egui::SidePanel::right("user_panel").show(ctx, |ui| {});

        egui::TopBottomPanel::bottom("my_panel").show(ctx, |ui| {
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
}