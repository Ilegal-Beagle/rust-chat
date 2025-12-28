use std::collections::HashMap;
use crate::user::user::User;

// handles user related content like the local user, other connected users
#[derive(Default)]
pub(crate) struct UserState {
    pub(crate) local: User,
    pub(crate) peers: HashMap<String, String>,
    pub(crate) profile_picture_list: Vec<String>,
}
