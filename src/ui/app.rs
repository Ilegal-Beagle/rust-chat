// ui.rs
use std::{
    collections::HashMap,
    fs::read,
    net::SocketAddr,
};
use egui::{RichText, vec2};
use egui_file_dialog::FileDialog;
use tokio::sync::mpsc::{channel, Sender, Receiver};
use crate::{
    message::{Disconnect, Handshake, MessageType, Notification},
    network::client::NetworkClient,
    tenor,
    user::User,
    gif,
};
use local_ip_address::local_ip;

enum View {
    Start,
    Connect,
    Chat,
}

pub struct App {
    pub(crate) network: NetworkState,
    pub(crate) user: UserState,
    pub(crate) ui: UiState,
    view: View,
    pub(crate) rt_handle: tokio::runtime::Handle,
    pub(crate) tenor_api: tenor::TenorAPI,
    pub(crate) gif_cache: HashMap<String, gif::Gif>,
}

pub(crate) struct NetworkState {
    pub(crate) tx: Sender<Vec<gif::Gif>>,
    pub(crate) rx: Receiver<Vec<gif::Gif>>,
    pub(crate) socket_addr: SocketAddr,
    pub(crate) bad_ip_msg: bool,
    pub(crate) ip_str: String,
    pub(crate) client: Option<NetworkClient>,
}

pub(crate) struct UserState {
    pub(crate) local: User,
    pub(crate) peers: HashMap<String, String>,
    pub(crate) profile_picture_list: Vec<String>,

}

pub(crate) struct UiState {
    pub(crate) message_text: String,
    pub(crate) gif_search_text: String,
    pub(crate) messages: Vec<MessageType>,
    pub(crate) file_dialog: FileDialog,
    pub(crate) image_bytes: Vec<u8>,
}

impl App {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        let (tx, rx) = channel::<Vec<gif::Gif>>(128);
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

        let net = NetworkState {
            tx:tx,
            rx: rx,
            socket_addr: sock,
            ip_str: sock.to_string(),
            bad_ip_msg: false,
            client: None,
        };

        let user = UserState {
            peers: HashMap::new(),
            local: User::new("Default".to_string(), Vec::new()),
            profile_picture_list: paths,
        };

        let ui = UiState {
            message_text: "".to_owned(),
            gif_search_text: "".to_owned(),
            messages: Vec::new(),
            file_dialog: FileDialog::new(),
            image_bytes: Vec::<u8>::new(),
        };

        Self {
            network: net,
            user: user,
            ui: ui,
            view: View::Start,
            rt_handle: handle,
            tenor_api: tenor,
            gif_cache: HashMap::new(),
        }
    }

    // rendering the chat state along with its UI components
    fn render_chat(&mut self, ctx: &egui::Context) {
        
        // recieve message network side
        if let Some(net) = &mut self.network.client {
            match net.recv() {
                Some(msg) => self.ui.messages.push(msg),
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
                ui.add(egui::TextEdit::singleline(&mut self.user.local.name).desired_width(100.0));
                ui.add_space(5.0);

                ui.label("ip and port: ");
                ui.add(egui::TextEdit::singleline(&mut self.network.ip_str).desired_width(100.0));
                ui.add_space(10.0);

                ui.heading(RichText::new("Choose a Profile Picture"));
                egui::Grid::new("profile_pictures").show(ui, |ui| {
                    for path in self.user.profile_picture_list.iter() {
                        let image = egui::Image::from_uri(path);
                        if ui.add(
                            egui::Button::image(image.fit_to_fraction(vec2(2.0, 2.0)))
                        ).clicked() {
                            let p = path.trim_start_matches("file://");
                            self.user.local.picture = read(p).unwrap(); // should never fail
                        }
                    }
                });

                ui.add_space(10.0);
                
                if ui.button("Enter").clicked()  {
                    match self.network.ip_str.as_str().parse() {
                        Ok(ip) => {
                            self.network.socket_addr = ip;
                            self.view = View::Connect;
                        },
                        Err(_) => {
                            self.network.bad_ip_msg = true;
                        },
                    }
                }

                if self.network.bad_ip_msg {
                    ui.colored_label(egui::Color32::DARK_RED, "Invalid IP chosen");
                }
            });
        });
    }

    // helper functions
    fn handle_connect(&mut self) {
        self.network.client = Some(NetworkClient::connect(self.network.socket_addr, &self.rt_handle));
        self.view = View::Chat;

        // messages to send
        let messages: Vec<MessageType> = vec![
            MessageType::Handshake(
                Handshake { user_name: self.user.local.name.clone()}
            ),
            MessageType::Notification(
                Notification { message: format!("{} has joined the chat", self.user.local.name)
            })
        ];
        
        // send messages
        if let Some(net) = &self.network.client {
            for message in messages {
                net.send(message.clone(), &self.rt_handle);
            }
        }
    }
    
    fn handle_disconnect(&mut self) {
        
        // messages to send
        let messages: Vec<MessageType> = vec![
            MessageType::Disconnect(
                Disconnect { user_name: self.user.local.name.clone(), ip: self.network.ip_str.clone()}
            ),
            MessageType::Notification(
                Notification {message: format!("{} has left the chat", self.user.local.name)}
            )
        ];

        // send messages
        if let Some(net) = &self.network.client {
            for message in messages {
                net.send(message, &self.rt_handle);
            }
        }
    }

}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.view {
            View::Start => self.render_start(ctx),
            View::Chat => self.render_chat(ctx),
            View::Connect => self.handle_connect(),
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.handle_disconnect();
    }
}
