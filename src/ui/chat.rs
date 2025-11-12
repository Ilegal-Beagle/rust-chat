// ui.rs
use crate::network;

use std::{
    thread,
    // io::{prelude::*, stdin},
    net::{TcpListener, TcpStream, SocketAddr, IpAddr, Ipv4Addr},
    time::{Duration},
    sync::{mpsc},
};



pub struct App {
    text: String,
    init: bool,
    messages: Vec<Message>,
    users: Vec<String>,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
}

struct Message {
    user_name: String,
    message: String,
}

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            text: "".to_owned(),
            init: true,
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
                let listener = TcpListener::bind(&SOCKET).expect("couldnt bind");
                println!("Started listening on port 7878");
                
                for stream in listener.incoming() {
                    let stream = stream.unwrap();
                    thread::spawn(|| {
                        let _ = network::handle_connection(stream);
                    });
                }
            } );
            

        } else {

            // create a client
            thread::spawn(move || {
                let mut stream = TcpStream::connect("127.0.0.1:7878").expect("could not connect");
                println!("connected to server.");

                loop {
                    // get client message from UI
                    match rx_ui.try_recv() {
                        Ok(msg) => println!("message received: {}", msg.message),
                        Err(_) => {},
                    }
                    // get server response
                }
            } );
        }

        Self {
            text: "".to_owned(),
            init: true,
            messages: Vec::new(),
            users: Vec::new(),
            tx: tx_ui,
            rx: rx_net,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::SidePanel::right("user_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                for msg in self.messages.iter_mut() {
                    ui.add(msg);
                }
            });
        });

        egui::TopBottomPanel::bottom("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let response = ui.text_edit_singleline(&mut self.text);

                // When enter is pressed in text box or send button is pressed
                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) ||
                    ui.button("send").clicked() {
                    
                    self.tx.send( Message {
                        user_name: "default".to_string(),
                        message: self.text.clone(),
                    } );

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