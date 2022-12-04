use serde::{Serialize, Deserialize};
use rmp_serde;


#[derive(Debug, Serialize, Deserialize)]
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
    t: Type,
    d: Option<Vec<u8>>,
}

impl Message {
    pub fn new_request() -> Self {
        Self { t: Type::Request, d: None }
    }

    pub fn new_accept() -> Self {
        Self { t: Type::Accept, d: None }
    }

    pub fn new_deny() -> Self {
        Self { t: Type::Deny, d: None }
    }

    pub fn new_confirm() -> Self {
        Self {t: Type::Confirm, d: None}
    }

    pub fn new_speak_plain(payload: Vec<u8>) -> Self {
        Self { t: Type::SpeakPlain, d: Some(payload) }
    }

    pub fn new_close() -> Self {
        Self {t: Type::Close, d: None}
    }

    // TODO: return error instead of expecting
    pub fn serialize(&self) -> Vec<u8> {
        rmp_serde::to_vec(&self).expect("Serialization successfull")
    }

    // TODO: return error instead of expecting
    pub fn deserialize(bytes: &[u8]) -> Self {
        rmp_serde::from_slice(bytes).expect("Deserialization successfull")
    }
}