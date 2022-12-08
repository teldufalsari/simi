use std::io::Cursor;
use std::path::PathBuf;
use std::fs;
use std::{io::Write, net::SocketAddr};
use std::net::TcpStream;

use rand::{thread_rng, Rng};
use rsa::{PublicKey, RsaPrivateKey, RsaPublicKey, PaddingScheme};
use aes_gcm::{
    aead::{KeyInit, Key, Aead},
    aes::Aes128, Aes128Gcm, Nonce
};
use image::RgbImage;

use crate::error::{Error, ErrCode, convert_err};
use crate::core::debug_prompt;

pub mod lsb;
pub mod message;
use message::{Message, Type, AcceptPayload, RandAndKey};

use self::message::RequestPayload;

#[derive(Debug)]
pub struct CryptoContext {
    pub peer_public_key: RsaPublicKey,
    pub session_key: Key<Aes128>,
    pub nonce: u64,
}

/// Write specified message into the stream
pub fn send(stream: &mut TcpStream, message: Message) -> Result<(), Error> {
    stream.write_all(
        message
        .serialize()?
        .as_slice()
    ).map_err(|e| convert_err(e, ErrCode::Network))
}


pub fn send_secret(stream: &mut TcpStream, port: u16, text: &str, path: PathBuf, key: &Key<Aes128>) -> Result<(), Error> {
    let img = try_load_image(path)?;
    let mut rng = rand::thread_rng();
    let mut raw_nonce = [0u8; 12];
    rng.fill(&mut raw_nonce);
    let nonce = Nonce::from_slice(&raw_nonce[..12]);
    let cipher = Aes128Gcm::new(key);
    let mut ciphertext = cipher.encrypt(nonce, text.as_ref()).unwrap();
    let mut payload = nonce.to_vec();
    payload.append(&mut ciphertext);
    let secret_image = lsb::embed(img, payload);
    let mut serialized_img: Vec<u8> = Vec::new();
    secret_image.write_to(&mut Cursor::new(&mut serialized_img), image::ImageOutputFormat::Png)
        .map_err(|e| convert_err(e, ErrCode::Serial))?;
    send(stream, Message::new_speak(port, serialized_img))?;
    Ok(())
}

fn try_load_image(supplied_path: PathBuf) -> Result<RgbImage, Error> {
    debug_prompt(&format!("path supplied: {}", supplied_path.display()));
    let path_to_img = if supplied_path.is_dir() {
        let entries = fs::read_dir(&supplied_path)
            .map_err(|e| convert_err(e, ErrCode::Filesys))?
            .filter_map(|x| x.ok())
            .filter(|x| x.file_name().to_str().unwrap().ends_with(".png"))
            .collect::<Vec<_>>();
        if entries.is_empty() {
            return Err(Error::new(
                ErrCode::Filesys,
                format!("no images found in {}", supplied_path.display())));
        }
        entries.first().unwrap().path()
    } else {
        supplied_path
    };
    debug_prompt(&format!("embedding secret into {}", path_to_img.display()));
    let img = image::open(path_to_img)
        .map_err(|e| convert_err(e, ErrCode::Filesys))?
        .to_rgb8();
    Ok(img)
}

pub fn decrypt_secret(secret: Vec<u8>, key: &Key<Aes128>) -> Result<String, Error> {
    let secret_image = image::load_from_memory_with_format(&secret, image::ImageFormat::Png)
        .map_err(|e| convert_err(e, ErrCode::Serial))?.to_rgb8();
    let payload = lsb::extract(secret_image)?;
    let cipher = Aes128Gcm::new(key);
    let nonce = Nonce::from_slice(&payload[..12]);
    let raw_text = cipher.decrypt(nonce, &payload[12..])
        .map_err(|e| convert_err(e, ErrCode::Serial))?;
    String::from_utf8(raw_text).map_err(|e| convert_err(e, ErrCode::Serial))
}


/// Read a message from the stream (if any)
pub fn recieve(stream: &mut TcpStream) -> Result<Message, Error> {
    Message::deserialize(stream)
}

/// Performs handshake and return `true` if connection has been established
/// In future it should return session parameters (key and nonce)
pub fn handshake_init(stream: &mut TcpStream, port: u16, private_key: &RsaPrivateKey) -> Result<Option<CryptoContext>, Error> {
    let mut rng = thread_rng();
    let public_key = RsaPublicKey::from(private_key);
    let padding = PaddingScheme::new_pkcs1v15_encrypt();

    debug_prompt("initializing handshake...");
    send(stream, Message::new_request(port, public_key))?;
    debug_prompt("reading response");
    let reply = Message::deserialize(stream)?;
    if reply.t == Type::Accept && reply.data.is_some() {
        let accept_data = AcceptPayload::deserialize(&reply.data.unwrap())?;
        let r_key = 
            RandAndKey::from_ciphertext(&private_key, padding, &accept_data.enc)?;
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let confirm_data = 
            accept_data.pkey.encrypt(&mut rng, padding, &r_key.serialize().unwrap()).unwrap();

        debug_prompt("accepted - sending confirmation");
        send(stream, Message::new_confirm(port, confirm_data))?;
        let ctx = CryptoContext {
            peer_public_key: accept_data.pkey,
            session_key: *Key::<Aes128>::from_slice(&r_key.session_key),
            nonce: r_key.nonce,
        };
        debug_prompt(&format!("Context: {:?}", ctx));
        Ok(Some(ctx))
    } else {
        debug_prompt("negative response. returning");
        Ok(None)
    }
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
    private_key: &RsaPrivateKey,
    mut connection: (TcpStream, SocketAddr),
    port: u16,
    desired: &SocketAddr
) -> Result<Option<CryptoContext>, Error> {
    let request = recieve(&mut connection.0)?;
    if request.t == Type::Request && request.data.is_some() {
        if request.port == desired.port() {
            debug_prompt(&format!("incoming connection from {} - accepting", desired));
            let mut rng = thread_rng();
            let public_key = RsaPublicKey::from(private_key);
            let padding = PaddingScheme::new_pkcs1v15_encrypt();
            let nonce = rng.gen::<u64>();
            let session_key = Aes128Gcm::generate_key(&mut rng);
            let peer_public_key = 
                RequestPayload::deserialize(&request.data.unwrap()).unwrap().pkey;
            let rand_and_key = peer_public_key.encrypt(
                &mut rng,
                padding,
                &RandAndKey {nonce, session_key: session_key.to_vec()}.serialize().unwrap())
                .unwrap();
            send(&mut connection.0, Message::new_accept(port, public_key, rand_and_key))?;
            let response = recieve(&mut connection.0)?;
            if response.t == Type::Confirm && response.data.is_some() {
                debug_prompt("acception confirmed");
                // check
                let padding = PaddingScheme::new_pkcs1v15_encrypt();
                let rand_and_key_check = 
                    RandAndKey::from_ciphertext(&private_key, padding, &response.data.unwrap())?;

                if rand_and_key_check.nonce != nonce
                    && rand_and_key_check.session_key.as_slice() != session_key.as_slice() {
                    return Err(Error::new(ErrCode::Network, "ill-formed request".to_owned()));
                }
                let ctx = CryptoContext {
                    peer_public_key,
                    session_key,
                    nonce,
                };
                debug_prompt(&format!("Context: {:?}", ctx));
                Ok(Some(ctx))
            } else {
                Err(Error::new(ErrCode::Network, "ill-formed request".to_owned()))
            }
        } else {
            debug_prompt(&format!("incoming connection from {}:{} - declining", connection.1.ip(), request.port));
            send(&mut connection.0, Message::new_deny(port))?;
            Ok(None)
        }
    } else {
        Err(Error::new(ErrCode::Network, "ill-formed request".to_owned()))
    }
}
