// ui.rs
use crate::{
    gif,
    message::{Disconnect, Handshake, MessageType, Notification},
    network::{client::NetworkClient, state::NetworkState},
    tenor,
    user::state::UserState,
    views::state::View,
};
use egui_file_dialog::FileDialog;
use local_ip_address::local_ip;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::mpsc::channel;

pub struct App {
    pub(crate) network: NetworkState,
    pub(crate) user: UserState,
    pub(crate) io: Io,
    pub(crate) env: Env,
    pub(crate) view: View,
    pub(crate) rt_handle: tokio::runtime::Handle,
    pub(crate) tenor_api: tenor::TenorAPI,
    pub(crate) gif_cache: HashMap<String, gif::Gif>,
}

// handles the input/output of app
#[derive(Default)]
pub(crate) struct Io {
    pub(crate) file_dialog: FileDialog,
    pub(crate) gif_search_text: String,
    pub(crate) image_bytes: Vec<u8>,
    pub(crate) message_text: String,
    pub(crate) messages: Vec<MessageType>,
}

#[derive(Default)]
pub(crate) struct Env {
    pub(crate) window_size: egui::Vec2,
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
            tx: tx,
            rx: rx,
            socket_addr: sock,
            ip_str: sock.to_string(),
            bad_ip_msg: false,
            client: None,
        };

        let mut user = UserState::default();
        user.profile_picture_list = paths;
        let io = Io::default();

        Self {
            network: net,
            user: user,
            io: io,
            env: Env::default(),
            view: View::Start,
            rt_handle: handle,
            tenor_api: tenor,
            gif_cache: HashMap::new(),
        }
    }

    fn handle_connect(&mut self) {
        self.network.client = Some(NetworkClient::connect(
            self.network.socket_addr,
            &self.rt_handle,
        ));
        self.view = View::Chat;

        // messages to send
        let messages: Vec<MessageType> = vec![
            MessageType::Handshake(Handshake {
                user_name: self.user.local.name.clone(),
            }),
            MessageType::Notification(Notification {
                message: format!("{} has joined the chat", self.user.local.name),
            }),
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
            MessageType::Disconnect(Disconnect {
                user_name: self.user.local.name.clone(),
                ip: self.network.ip_str.clone(),
            }),
            MessageType::Notification(Notification {
                message: format!("{} has left the chat", self.user.local.name),
            }),
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
        let rect = ctx.content_rect();
        self.env.window_size = egui::vec2(rect.width(), rect.height());

        match self.view {
            View::Start => self.render_start(ctx),
            View::Chat => self.render_chat(ctx),
            View::Connect => self.handle_connect(),
            View::Select => self.render_select(ctx),
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.handle_disconnect();
    }
}
