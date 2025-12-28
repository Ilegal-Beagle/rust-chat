use tokio::sync::mpsc::{Sender, Receiver};
use std::net::SocketAddr;
use crate::{network::client::NetworkClient, gif};

// handles anything network related
pub(crate) struct NetworkState {
    pub(crate) tx: Sender<Vec<gif::Gif>>,
    pub(crate) rx: Receiver<Vec<gif::Gif>>,
    pub(crate) socket_addr: SocketAddr,
    pub(crate) bad_ip_msg: bool,
    pub(crate) ip_str: String,
    pub(crate) client: Option<NetworkClient>,
}
