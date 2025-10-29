use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerBoundPackets {
    JoinRequest {
        username: Option<String>,
        password_hash: String
    },
    SendMessage {
        message: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientBoundPackets {
    JoinSuccess,
    JoinError {
        what: String,
    },
    NewMessage {
        author: String,
        contents: String,
    }
}

pub fn hash_password(password: String) -> String {
    //TODO: bcrypt argus or something idk.
    password
}