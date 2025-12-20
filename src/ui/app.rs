// ui.rs
use std::{
    collections::HashMap,
    fs::read,
    net::{SocketAddr},
};
use egui::{RichText, vec2};
use egui_file_dialog::FileDialog;
use crate::{
    message::{Disconnect, Handshake, MessageType, Notification},
    network::{client::NetworkClient},
};
use local_ip_address::local_ip;
use crate::tenor;

enum State {
    Start,
    Connect,
    Chat,
}

pub struct App {
    pub(crate) user_name: String,
    pub(crate) profile_picture: Vec::<u8>,
    pub(crate) text: String,
    pub(crate) messages: Vec<MessageType>,
    pub(crate) users: HashMap<String, String>,
    current_state: State,
    pub(crate) file_dialog: FileDialog,
    pub(crate) socket_addr: SocketAddr,
    pub(crate) ip_str: String,
    pub(crate) image_bytes: Vec<u8>,
    pub(crate) profile_picture_list: Vec<String>,
    pub(crate) bad_ip_msg: bool,
    pub(crate) rt_handle: tokio::runtime::Handle,
    pub(crate) tenor_api: tenor::TenorAPI,
    pub(crate) client: Option<NetworkClient>,
    pub(crate) _gif_cache: Vec<String>,
}

impl App {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        let sock = SocketAddr::new(local_ip().unwrap(), 5000);
        let tenor = match tenor::TenorAPI::new() {
            Ok(api) => api,
            Err(_) => panic!(),
        };
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
            current_state: State::Start,
            file_dialog: FileDialog::new(),
            socket_addr: sock,
            ip_str: sock.to_string(),
            image_bytes: Vec::<u8>::new(),
            profile_picture_list: paths,
            bad_ip_msg: false,
            rt_handle: handle,
            tenor_api: tenor,
            client: None,
            _gif_cache: Vec::<String>::new(),
        }
    }

    // rendering the chat state along with its UI components
    fn render_chat(&mut self, ctx: &egui::Context) {
        
        // recieve message network side
        if let Some(net) = &mut self.client {
            match net.recv() {
                Some(msg) => self.messages.push(msg),
                None => {},
            }
        }

        // render ui
        self.message_panel(ctx);

        self.side_panel(ctx);

        self.chat_panel(ctx);
    }


    // rendering the start state UI
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

    // helper functions
    fn handle_connect(&mut self) {
        self.client = Some(NetworkClient::connect(self.socket_addr, &self.rt_handle));
        self.current_state = State::Chat;

        // messages to send
        let messages: Vec<MessageType> = vec![
            MessageType::Handshake(
                Handshake { user_name: self.user_name.clone()}
            ),
            MessageType::Notification(
                Notification { message: format!("{} has joined the chat", self.user_name)
            })
        ];
        
        // send messages
        if let Some(net) = &self.client {
            for message in messages {
                net.send(message.clone(), &self.rt_handle);
            }
        }
    }
    
    fn handle_disconnect(&mut self) {
        
        // messages to send
        let messages: Vec<MessageType> = vec![
            MessageType::Disconnect(
                Disconnect { user_name: self.user_name.clone(), ip: self.ip_str.clone()}
            ),
            MessageType::Notification(
                Notification {message: format!("{} has left the chat", self.user_name)}
            )
        ];

        // send messages
        if let Some(net) = &self.client {
            for message in messages {
                net.send(message, &self.rt_handle);
            }
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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.handle_disconnect();
    }
}
