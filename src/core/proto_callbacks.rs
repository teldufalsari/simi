use std::io::Write;
use std::net::TcpStream;

use crate::core::proto::{Message, Type};
use crate::error::{Error, ErrCode, convert_err};

pub fn send(stream: &mut TcpStream, message: Message) -> Result<(), Error> {
    stream.write_all(
        message
        .serialize()?
        .as_slice()
    ).map_err(|e| convert_err(e, ErrCode::Network))
}

/// Performs handshake and return `true` if connection has been established
/// In future it should return session parameters (key and nonce)
pub fn handshake_init(stream: &mut TcpStream) -> Result<bool, Error> {
    send(stream, Message::new_request())?;
    let reply = Message::deserialize(stream)?;
    Ok(if reply.t == Type::Accept {true} else {false})
}