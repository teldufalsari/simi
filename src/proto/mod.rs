use std::io::Write;
use std::net::TcpStream;

use crate::error::{Error, ErrCode, convert_err};

pub mod message;
use message::{Message, Type};

/// Write specified message into the stream
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


/// Try recieveng message; if it's a valid request,
/// a decline message is sent back.
pub fn decline(mut stream: TcpStream) {
    // TODO error handling
    if let Ok(msg) = Message::deserialize(&mut stream) {
        if msg.t == Type::Request {
            send(&mut stream, Message::new_deny()).unwrap();
        }
    }
}
