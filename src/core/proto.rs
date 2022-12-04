use serde::{Serialize, Deserialize};
use rmp_serde;


#[derive(Debug, Serialize, Deserialize)]
enum Type {
    Request,
    Accept,
    Deny,
    Confirm,
    Speak,
    SpeakPlain,
    Close,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    t: Type,
    d: Option<Vec<u8>>,
}

impl Message {
    fn new_request() -> Self {
        Self { t: Type::Request, d: None }
    }

    fn new_accept() -> Self {
        Self { t: Type::Accept, d: None }
    }

    fn new_deny() -> Self {
        Self { t: Type::Deny, d: None }
    }

    fn new_confirm() -> Self {
        Self {t: Type::Confirm, d: None}
    }

    fn new_speak_plain(payload: Vec<u8>) -> Self {
        Self { t: Type::SpeakPlain, d: Some(payload) }
    }

    fn new_close() -> Self {
        Self {t: Type::Close, d: None}
    }

    fn serialize(&self) -> Vec<u8> {
        rmp_serde::to_vec(&self).expect("Serialization successfull")
    }
}