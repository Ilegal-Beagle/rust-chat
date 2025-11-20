// ui.rs
use crate::network;
use crate::message::Message;

use std::{
    thread,
    fs::{read},
    net::{SocketAddr, IpAddr, Ipv4Addr},
    time::{Duration},
    sync::{mpsc},
};

use egui::{RichText};
use egui_file_dialog::FileDialog;

enum State {
    Start,
    Connect,
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
    socket_addr: SocketAddr,
    ip_str: String,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            user_name: "Default".to_string(),
            text: "".to_owned(),
            messages: Vec::new(),
            users: Vec::new(),
            tx: tx,
            rx: rx,
            current_state: State::Start,
            file_dialog: FileDialog::new(),
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 7878),
            ip_str: "127.0.0.1:7878".to_string(),
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
                            image: Vec::<u8>::new(),
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
                    let p = path.to_str().unwrap();
                    let image = read(p).expect("invalid file path");

                    self.tx
                    .send(Message {
                        user_name: self.user_name.clone(),
                        message: self.text.clone(),
                        image: image,
                    })
                    .unwrap();
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
            ui.heading("pibbles");
            egui::ScrollArea::vertical()
            .enable_scrolling(true)
            .stick_to_bottom(true)
            .auto_shrink(false)
            .show(ui, |ui| {
                for msg in self.messages.iter_mut() {
                    ui.add(msg);
                }
            });
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

                ui.horizontal(|ui| {
                    ui.label("ip and port: ");
                    ui.text_edit_singleline(&mut self.ip_str);
                });

                if ui.button("Enter").clicked()  {
                    self.socket_addr = self.ip_str.as_str().parse().expect("cant");
                    println!("{:?}", self.socket_addr);
                    self.current_state = State::Connect;
                }
            });
        });
    }

    fn handle_connect(&mut self) {
        const TIMEOUT: Duration = Duration::new(5, 0);

        let (tx_ui, rx_ui) = mpsc::channel::<Message>();
        let (tx_net, rx_net) = mpsc::channel::<Message>();
        let socket = self.socket_addr.clone();
        
        // if no server is found
        if !network::try_connect(&socket, TIMEOUT) {
            thread::spawn( move || {
                let _ = network::server(&socket);
            });
        }

        thread::sleep(Duration::new(1, 0));
        thread::spawn(move || {
            let _ = network::client(&socket, rx_ui, tx_net);
        });

        self.tx = tx_ui;
        self.rx = rx_net;
        self.current_state = State::Chat;
    }

}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.current_state {
            State::Start => self.render_start(ctx),
            State::Chat => self.render_chat(ctx),
            State::Connect => self.handle_connect(),
        }
    }
}
