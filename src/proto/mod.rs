use std::{io::Write, net::SocketAddr};
use std::net::TcpStream;

use crate::error::{Error, ErrCode, convert_err};
use crate::core::prompt;

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

/// Read a message from the stream (if any)
pub fn recieve(stream: &mut TcpStream) -> Result<Message, Error> {
    Message::deserialize(stream)
}

/// Performs handshake and return `true` if connection has been established
/// In future it should return session parameters (key and nonce)
pub fn handshake_init(stream: &mut TcpStream, port: u16) -> Result<bool, Error> {
    send(stream, Message::new_request(port))?;
    let reply = Message::deserialize(stream)?;
    Ok(if reply.t == Type::Accept {true} else {false})
}


/// Try recieveng message; if it's a valid request,
/// a decline message is sent back.
pub fn decline(mut stream: TcpStream, port: u16) {
    // TODO error handling
    if let Ok(msg) = Message::deserialize(&mut stream) {
        if msg.t == Type::Request {
            send(&mut stream, Message::new_deny(port)).unwrap();
        }
    }
}

/// Try recieving a message from `connection`; if it's a valid request,
/// a valid response is sent.
/// 
/// Returns `true` if the request was accepted, `false` otherwise.
pub fn accept_or_decline(
    mut connection: (TcpStream, SocketAddr),
    port: u16,
    desired: &SocketAddr
) -> Result<bool, Error> {
    let msg = recieve(&mut connection.0);
    if let Ok(request) = msg {
        if request.t == Type::Request {
            if request.port == desired.port() {
                prompt(&format!("incoming connection from {} - accepting", desired));
                send(&mut connection.0, Message::new_accept(port))?;
                Ok(true)
            } else {
                prompt(&format!("incoming connection from {}:{} - declining", connection.1.ip(), request.port));
                send(&mut connection.0, Message::new_deny(port))?;
                Ok(false)
            }
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}
