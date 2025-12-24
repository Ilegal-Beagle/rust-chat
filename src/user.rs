pub struct User {
    pub(crate) name: String,
    pub(crate) picture: Vec::<u8>,
}

impl User {
    pub fn new(name: String, picture: Vec::<u8>) -> Self {
        Self {
            name: name,
            picture: picture,
        }
    }
}