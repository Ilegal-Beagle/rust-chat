#[derive(Clone)]
#[allow(dead_code)]
pub struct Gif {
    pub(crate) id: String,
    pub(crate) url: String,
    pub(crate) tinygif_url: String,
    pub(crate) gif_bytes: Vec<u8>,
}
