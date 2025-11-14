// ui.rs
use crate::network;

use std::{
    thread,
    fmt,
    io::{prelude::*},
    net::{TcpListener, TcpStream, SocketAddr, IpAddr, Ipv4Addr},
    time::{Duration},
    sync::{mpsc},
};

use serde::{Serialize, Deserialize};
use serde_json;

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
                let mut stream = TcpStream::connect(&SOCKET).expect("could not connect");
                stream.set_read_timeout(Some(Duration::new(0, 10000))).expect("could not set timeout");
                let mut buf = [0u8; 128];
                println!("connected to server.");

                loop {
                    // get client message from UI
                    match rx_ui.try_recv() {
                        
                        Ok(msg) => {
                            // serialize message
                            let serialized_msg = match serde_json::to_string(&msg) {
                                Ok(msg) => msg,                                
                                Err(e) => {
                                    eprintln!("error in serializing message: {e:?}");
                                    continue;
                                },
                            };
                            
                            // send message to server
                            match stream.write_all(serialized_msg.as_bytes()) {
                                Ok(_) => {println!("message sent to server")},
                                Err(e) => {println!("Error in sending message: {e:?}")},
                            };

                            // receive repsonse from server
                            match stream.read(&mut buf) {
                                Ok(0) => break,

                                Ok(bytes_read) => {
                                    // UTF8 to str
                                    let message = match str::from_utf8(&buf[..bytes_read]) {
                                        Ok(msg) => msg,
                                        Err(e) => {
                                            eprintln!("Invalid UTF8: {e}");
                                            continue;
                                        },
                                    };

                                    // str to Message
                                    let deserialized_msg: Message = match serde_json::from_str(&message) {
                                        Ok(msg) => msg,
                                        Err(e) => {
                                            eprintln!("Could not deserialize message: {e}");
                                            continue;
                                        },
                                    };

                                    match tx_net.send(deserialized_msg) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            eprintln!("Error sending message: {e}");
                                            continue;
                                        }
                                    }

                                },

                                Err(e) => {
                                    eprintln!("Error receiving message: {e}");
                                    continue;
                                },

                            };

                        },

                        Err(_) => {/* Do nothing if nothing was recved */},
                    }
                    
                }
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