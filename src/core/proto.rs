use std::io::Read;
use serde::{Serialize, Deserialize};
use bincode::{self, Options};

use crate::error::{Error, ErrCode, convert_err};

pub const HEADER_LEN: usize = 8;
pub const MSG_LIMIT: u64 = 1024 * 1024 * 16; // 16 MiB


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Type {
    Request,
    Accept,
    Deny,
    Confirm,
    Speak,
    SpeakPlain,
    Close,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub t: Type,
    pub data: Option<Vec<u8>>,
}

impl Message {
    pub fn new_request() -> Self {
        Self { t: Type::Request, data: None }
    }

    pub fn new_accept() -> Self {
        Self { t: Type::Accept, data: None }
    }

    pub fn new_deny() -> Self {
        Self { t: Type::Deny, data: None }
    }

    pub fn new_confirm() -> Self {
        Self {t: Type::Confirm, data: None}
    }

    pub fn new_speak_plain(payload: Vec<u8>) -> Self {
        Self { t: Type::SpeakPlain, data: Some(payload) }
    }

    pub fn new_close() -> Self {
        Self {t: Type::Close, data: None}
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Error> {
        bincode::DefaultOptions::new()
            .with_little_endian()
            .with_limit(MSG_LIMIT)
            .serialize(self)
            .map_err(|e| convert_err(e, ErrCode::Serial))
    }

    pub fn deserialize<R: Read>(reader: R) -> Result<Self, Error> {
        bincode::DefaultOptions::new()
            .with_little_endian()
            .with_limit(MSG_LIMIT)
            .deserialize_from(reader)
            .map_err(|e| convert_err(e, ErrCode::Serial))
    }
}