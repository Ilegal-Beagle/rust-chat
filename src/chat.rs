// ui.rs
use std::{
    thread,
    fs::{read},
    net::{SocketAddr},
    time::{Duration},
    sync::{mpsc},
    collections::{HashMap},
};
use egui::{Align, Layout, RichText, vec2};
use egui_file_dialog::FileDialog;
use uuid::Uuid;
use crate::network::{helpers, server, client};
use crate::message::{MessageType, Message, Handshake};
use local_ip_address::local_ip;


enum State {
    Start,
    Connect,
    Chat,
}

pub struct App {
    user_name: String,
    profile_picture: Vec::<u8>,
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
    profile_picture_list: Vec<String>,
    bad_ip_msg: bool,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let sock = SocketAddr::new(local_ip().unwrap(), 5000);
        let paths = vec![
            "file://assets/2000c.png".to_string(),
            "file://assets/20002.png".to_string(),
            "file://assets/20003.png".to_string(),
            "file://assets/20006.png".to_string(),
            "file://assets/20007.png".to_string(),
            "file://assets/20008.png".to_string(),
            "file://assets/21019.png".to_string(),
            "file://assets/21042.png".to_string(),
        ];
        Self {
            user_name: "Default".to_string(),
            profile_picture: Vec::new(),
            text: "".to_owned(),
            messages: Vec::new(),
            users: HashMap::new(),
            tx: tx,
            rx: rx,
            current_state: State::Start,
            file_dialog: FileDialog::new(),
            socket_addr: sock,
            ip_str: sock.to_string(),
            image_bytes: Vec::<u8>::new(),
            profile_picture_list: paths,
            bad_ip_msg: false,
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
                            profile_picture: self.profile_picture.clone(),
                            message: self.text.clone(),
                            image: self.image_bytes.clone(),
                            timestamp: time,
                            uuid: Uuid::new_v4().to_string(),
                            uuid_profile_picture: Uuid::new_v4().to_string(),
                        }))
                        .unwrap();

                    self.text.clear();
                    self.image_bytes.clear();
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

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    let gif_resp = ui.button("gifs");
                    let emoji_resp = ui.button("emojis");

                    if gif_resp.clicked() {
                        todo!();
                    };

                    // emoji popup menu
                    egui::Popup::menu(&emoji_resp)
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
                                            if ui.button(emoji.to_string()).clicked() {
                                                self.text.push(emoji);
                                            }
                                        }
                                });
                            });
                    });
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
                ui.separator();

                ui.label("Username: ");
                ui.add(egui::TextEdit::singleline(&mut self.user_name).desired_width(100.0));
                ui.add_space(5.0);

                ui.label("ip and port: ");
                ui.add(egui::TextEdit::singleline(&mut self.ip_str).desired_width(100.0));
                ui.add_space(10.0);

                ui.heading(RichText::new("Choose a Profile Picture"));
                egui::Grid::new("profile_pictures").show(ui, |ui| {
                    for path in self.profile_picture_list.iter() {
                        let image = egui::Image::from_uri(path);
                        if ui.add(
                            egui::Button::image(image.fit_to_fraction(vec2(2.0, 2.0)))
                        ).clicked() {
                            let p = path.trim_start_matches("file://");
                            self.profile_picture = read(p).unwrap(); // should never fail
                        }
                    }
                });

                ui.add_space(10.0);
                
                if ui.button("Enter").clicked()  {
                    match self.ip_str.as_str().parse() {
                        Ok(ip) => {
                            self.socket_addr = ip;
                            self.current_state = State::Connect;
                        },
                        Err(_) => {
                            self.bad_ip_msg = true;
                        },
                    }
                }

                if self.bad_ip_msg {
                    ui.colored_label(egui::Color32::DARK_RED, "Invalid IP chosen");
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
