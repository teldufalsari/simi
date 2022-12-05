use std::io::Read;
use serde::{Serialize, Deserialize};
use bincode::{self, Options};

use crate::error::{Error, ErrCode, convert_err};

/// Message length that must not be exceeded.
/// All incoming messages will be discarded if they are
/// longer.
pub const MSG_LIMIT: u64 = 1024 * 1024 * 16; // 16 MiB


/// Protocol message type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Type {
    Request,
    Accept,
    Deny,
    Confirm,
    Speak,
    SpeakPlain,
    Close,
}


/// A single protocol message
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub t: Type,
    pub data: Option<Vec<u8>>,
}

impl Message {
    /// Creates an empty request message
    pub fn new_request() -> Self {
        Self { t: Type::Request, data: None }
    }

    /// Creates an empty request message
    pub fn new_accept() -> Self {
        Self { t: Type::Accept, data: None }
    }

    /// Creates an empty request message
    pub fn new_deny() -> Self {
        Self { t: Type::Deny, data: None }
    }

    /// Creates an empty request message
    pub fn new_confirm() -> Self {
        Self {t: Type::Confirm, data: None}
    }

    /// Creates an empty request message
    pub fn new_speak_plain(payload: Vec<u8>) -> Self {
        Self { t: Type::SpeakPlain, data: Some(payload) }
    }

    /// Creates an empty request message
    pub fn new_close() -> Self {
        Self {t: Type::Close, data: None}
    }

    /// Serializes the message so that it can be sent.
    /// 
    /// Total length must not exceed `MSG_LIMIT` constant.
    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        bincode::DefaultOptions::new()
            .with_little_endian()
            .with_limit(MSG_LIMIT)
            .serialize(self)
            .map_err(|e| convert_err(e, ErrCode::Serial))
    }

    /// Attempts to read a message from the given reader (usually, a TCP socket)
    /// with memory exhaustion protection.
    pub fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        bincode::DefaultOptions::new()
            .with_little_endian()
            .with_limit(MSG_LIMIT)
            .deserialize_from(reader)
            .map_err(|e| convert_err(e, ErrCode::Serial))
    }
}