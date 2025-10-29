#[cfg(feature = "stream")]
pub mod stream;

use bincode::error::{DecodeError, EncodeError};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub enum Packet {
    ServerBound(ServerBoundPackets),
    ClientBound(ClientBoundPackets),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerBoundPackets {
    JoinRequest {
        username: Option<String>,
        password_hash: String,
    },
    SendMessage {
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientBoundPackets {
    JoinSuccess,
    JoinError { what: String },
    NewMessage { author: String, contents: String },
}

pub fn hash_password(password: String) -> String {
    //TODO: bcrypt argus or something idk.
    password
}

fn bincode_options() -> impl bincode::config::Config {
    bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding()
}

pub fn read_packet(read: &mut impl Read) -> Result<Packet, DecodeError> {
    bincode::serde::decode_from_std_read(read, bincode_options())
}

pub fn write_packet(pk: Packet, dest: &mut impl Write) -> Result<usize, EncodeError> {
    bincode::serde::encode_into_std_write(pk, dest, bincode_options())
}
