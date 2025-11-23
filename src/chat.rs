// ui.rs
use std::{
    thread,
    fs::{read},
    net::{SocketAddr, IpAddr, Ipv4Addr},
    time::{Duration},
    sync::{mpsc},
    collections::{HashMap},
};
use egui::{RichText};
use egui_file_dialog::FileDialog;
use uuid::Uuid;
use crate::network::{helpers, server, client};
use crate::message::{MessageType, Message, Handshake};

enum State {
    Start,
    Connect,
    Chat,
}

pub struct App {
    user_name: String,
    text: String,
    messages: Vec<MessageType>,
    users: HashMap<String, String>,
    tx: mpsc::Sender<MessageType>,
    rx: mpsc::Receiver<MessageType>,
    current_state: State,
    file_dialog: FileDialog,
    socket_addr: SocketAddr,
    ip_str: String,
    image_bytes: Vec<u8>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            user_name: "Default".to_string(),
            text: "".to_owned(),
            messages: Vec::new(),
            users: HashMap::new(),
            tx: tx,
            rx: rx,
            current_state: State::Start,
            file_dialog: FileDialog::new(),
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 7878),
            ip_str: "127.0.0.1:7878".to_string(),
            image_bytes: Vec::<u8>::new(),
        }
    }

    fn render_chat(&mut self, ctx: &egui::Context) {
        
        match self.rx.try_recv() {
            Ok(msg) => {self.messages.push(msg);},
            Err(_) => {}
        }

        egui::TopBottomPanel::bottom("message_entry").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let text_resp = ui.add(egui::TextEdit::singleline(&mut self.text)
                    .desired_width(250.0)
                    .hint_text("Type Here")
                );
                let send_button_resp = ui.button("send");
                let image_button_resp= ui.button("add image");

                // image handling
                if image_button_resp.clicked() {
                    self.file_dialog.pick_file();
                }

                self.file_dialog.update(ctx);

                if let Some(path) = self.file_dialog.take_picked() {
                    self.image_bytes = read(path.to_str().unwrap()).expect("invalid file path");
                }

                // When enter is pressed in text box or send button is pressed
                if (text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || send_button_resp.clicked()
                {
                    let time = chrono::Local::now().format("%I:%M %p").to_string();
                    self.tx
                        .send(MessageType::Message(Message {
                            user_name: self.user_name.clone(),
                            message: self.text.clone(),
                            image: self.image_bytes.clone(),
                            timestamp: time,
                            uuid: Uuid::new_v4().to_string(),
                        }))
                        .unwrap();

                    self.text.clear();
                }


            });
        });


        egui::SidePanel::right("user_panel")
            .resizable(false)
            .exact_width(100.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading(egui::RichText::new("Users"));
                    ui.separator();

                    for (key, _) in &mut self.users {
                        ui.label(key);
                    }
                });
            });


        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chat Room");
            ui.separator();
            egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink(false)
            .show(ui, |ui| {
                for msg in self.messages.iter_mut() {
                    match msg {
                        MessageType::Message(msg) => {ui.add(msg);},
                        MessageType::Notification(msg) => {ui.add(msg);},
                        MessageType::Handshake(_) => {},
                        MessageType::UserList(msg) => {
                            self.users = msg.clone();
                        },
                    }
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
                    ui.add(egui::TextEdit::singleline(&mut self.user_name).desired_width(100.0));
                });

                ui.horizontal(|ui| {
                    ui.label("ip and port: ");
                    ui.add(egui::TextEdit::singleline(&mut self.ip_str).desired_width(100.0));
                });

                if ui.button("Enter").clicked()  {
                    self.socket_addr = self.ip_str.as_str().parse().expect("cant");
                    self.current_state = State::Connect;
                }
            });
        });
    }

    fn handle_connect(&mut self) {
        const TIMEOUT: Duration = Duration::new(5, 0);

        let (tx_ui, rx_ui) = mpsc::channel::<MessageType>();
        let (tx_net, rx_net) = mpsc::channel::<MessageType>();
        let socket = self.socket_addr.clone();
        
        // if no server is found
        if !helpers::try_connect(&socket, TIMEOUT) {
            thread::spawn( move || {
                let _ = server::server(&socket);
            });
        }

        thread::sleep(Duration::from_millis(50));
        thread::spawn(move || {
            let _ = client::client(&socket, rx_ui, tx_net);
        });

        
        self.tx = tx_ui;
        self.rx = rx_net;
        self.current_state = State::Chat;

        thread::sleep(Duration::new(1, 0));
        match self.tx.send(MessageType::Handshake(Handshake { user_name: self.user_name.clone()})) {
            Ok(_) => {},
            Err(_) => {},
        }
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
