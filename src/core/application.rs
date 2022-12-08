use std::net::{TcpListener, TcpStream, SocketAddr};
use std::os::unix::prelude::{AsFd, AsRawFd};
use std::io::stdin;
use std::time::Duration;

use bincode::Options;
use nix::libc::STDIN_FILENO;
use nix::poll::{PollFd, PollFlags, poll};
use nix::errno::Errno;
use rsa::RsaPrivateKey;
use rand::thread_rng;


use crate::config::Config;
use crate::cli::{menu, dialogue, Command};
use crate::error::{Error, ErrCode, convert_err};
use crate::proto::message::{Type, Message};
use crate::proto::{handshake_init, decline, recieve, accept_or_decline, send};
use crate::proto::CryptoContext;
use super::{prompt, empty_prompt, named_prompt, debug_prompt};

pub struct Application {
    cfg: Config,
    addr: SocketAddr,
    listener: TcpListener,
    watches: [PollFd; 2],
    private_key: RsaPrivateKey,
}

#[derive(PartialEq)]
enum CloseCaused {
    ByRemote,
    Locally,
}

impl Application {
    pub fn initialize(cfg: Config) -> Result<Self, Error> {
        let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
        let listener = TcpListener::bind(addr)
            .map_err(|e| convert_err(e, ErrCode::Fatal))?;
        let listener_fd = listener.as_fd().as_raw_fd();
        let watches = [
            PollFd::new(STDIN_FILENO, PollFlags::POLLIN),
            PollFd::new(listener_fd, PollFlags::POLLIN)];
        
        let bits = 2048;
        let mut rng = thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();

        Ok(Self { cfg, addr, listener, watches, private_key})
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let mut buffer = String::new();
        prompt(&format!("listening on port {}", self.cfg.port));
        loop {
            match poll(&mut self.watches, -1) {
                Ok(0) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => continue,
                Ok(val) => val,
                Err(e) => return Err(Error::new(ErrCode::Fatal, e.to_string())),
            };
            if self.watches[0].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
                // Events in stdio
                // read line, interpret, execute
                stdin().read_line(&mut buffer).unwrap();
                match menu::interpret(buffer.trim()) {
                    Err(e) => prompt(&e.descr),
                    Ok(Command::Exit) => break,
                    Ok(cmd) => self.menu_execute(cmd),
                }
                buffer.clear();
            }
            if self.watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
                // TCP connection recieved, decline it
                let connection = self.listener.accept()
                    .map_err(|e| Error::new(ErrCode::Fatal, e.to_string()))?;
                decline(connection.0, self.cfg.port);
            }
        }
        Ok(())
    }

    fn waiting_loop(&mut self, desired_addr: SocketAddr, name: &str) -> Result<(), Error> {
        // First try connecting to the remote peer
        debug_prompt("dialing...");
        let ten_sec = Duration::from_secs(10);
        let mut stream = TcpStream::connect_timeout(&desired_addr, ten_sec)
            .map_err(|e| Error::new(ErrCode::Network, e.to_string()))?;
        stream.set_write_timeout(Some(ten_sec)).unwrap();
        stream.set_read_timeout(Some(ten_sec)).unwrap();
        if let Some(ctx) = handshake_init(&mut stream, self.cfg.port)? {
            // On success - wait until this connection is closed
            let cause = self.connected_loop(desired_addr, name, ctx)?;
            if cause == CloseCaused::Locally {
                // if it was closed by the local user - return to the idle loop,
                // otherwise go to the wait loop
                return Ok(());
            }
        } else {
            prompt("your peer is offline. Wait until they connect or leave");
        }
        let mut buffer = String::new();
        loop {
            match poll(&mut self.watches, -1) {
                Ok(0) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => continue,
                Ok(val) => val,
                Err(e) => return Err(Error::new(ErrCode::Fatal, e.to_string())),
            };
            if self.watches[0].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
                // Events in stdio
                // read line, interpret, execute
                stdin().read_line(&mut buffer).unwrap();
                match dialogue::interpret(buffer.trim()) {
                    Err(e) => prompt(&e.descr),
                    Ok(Command::Exit) => break,
                    Ok(cmd) => self.waiting_execute(cmd),
                }
                buffer.clear();
            }
            if self.watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
                // TCP connection recieved, decide on it
                let connection = self.listener.accept()
                    .map_err(|e| convert_err(e, ErrCode::Fatal))?;
                match accept_or_decline(&self.private_key, connection, self.cfg.port, &desired_addr) {
                    Ok(Some(ctx)) => {
                        let cause = self.connected_loop(desired_addr, name, ctx)?;
                        if cause == CloseCaused::Locally {
                            return Ok(());
                        }
                    }
                    Ok(None) | Err(_) => continue,
                    // Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    fn connected_loop(&mut self, address: SocketAddr, name: &str, ctx: CryptoContext) -> Result<CloseCaused, Error> {
        prompt("connected to the peer");
        let mut buffer = String::new();
        loop {
            match poll(&mut self.watches, -1) {
                Ok(0) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => continue,
                Ok(val) => val,
                Err(e) => return Err(Error::new(ErrCode::Fatal, e.to_string())),
            };
            if self.watches[0].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
                stdin().read_line(&mut buffer).unwrap();
                match dialogue::interpret(buffer.trim()) {
                    Err(e) => {
                        prompt(&e.descr);
                    }
                    Ok(Command::Exit) => {
                        let mut stream = TcpStream::connect(address)
                            .map_err(|e| convert_err(e, ErrCode::Network))?;
                        send(&mut stream, Message::new_close(self.cfg.port, ctx.nonce))?;
                        return Ok(CloseCaused::Locally)
                    }
                    Ok(cmd) => self.dialogue_execute(cmd, &address)?
                }
                buffer.clear();
            }
            if self.watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
                let connection = self.listener.accept()
                    .map_err(|e| convert_err(e, ErrCode::Fatal))?;
                    if let Ok(true) = self.handle_incoming_connection(connection, &address, name, &ctx) {
                        return Ok(CloseCaused::ByRemote);
                    }
            }
        }
    }

    fn menu_execute(&mut self, cmd: Command) {
        match cmd {
            Command::List => {
                if self.cfg.contacts.is_empty() {
                    prompt("empty")
                } else {
                    for contact in self.cfg.contacts.iter() {
                        println!("{} : {}", contact.0, contact.1);
                    }
                    empty_prompt()
                }
            }
            Command::Add(alias, addr) => {
                self.cfg.contacts.insert(alias, addr);
                empty_prompt()
            }
            Command::Remove(alias) => {
                if let None = self.cfg.contacts.remove(&alias) {
                    prompt(&format!("alias {} not found", alias))
                } else {
                    empty_prompt()
                }
            }
            Command::Save => {
                if let Err(e) = self.cfg.save() {
                    prompt(&format!("cannot save config: {}", e))
                } else {
                    empty_prompt()
                }
            }
            Command::DialIp(ip) => {
                let addr = ip.parse::<SocketAddr>().unwrap();
                self.dial(addr, &ip)
            }
            Command::DialAlias(alias) => {
                let ip = match self.cfg.contacts.get(&alias) {
                    Some(val) => val,
                    None => {
                        prompt(&format!("alias {} not found", alias));
                        return;
                    }
                };
                let addr = ip.parse::<SocketAddr>().unwrap();
                self.dial(addr, &alias)
            }
            _ => {}
        }
    }

    fn dialogue_execute(&mut self, cmd: Command, addr: &SocketAddr) -> Result<(), Error> {
        match cmd {
            Command::SpeakPlain(text) => {
                let mut stream = TcpStream::connect(addr)
                    .map_err(|e| convert_err(e, ErrCode::Network))?;
                send(&mut stream, Message::new_speak_plain(self.cfg.port, text.into_bytes()))?;
                empty_prompt();
            }
            Command::Secret(path) => debug_prompt(&format!(" secret {:?} not implemented", path)),
            _ => {}
        }
        Ok(())
    }

    fn waiting_execute(&mut self, cmd: Command) {
        match cmd {
            Command::SpeakPlain(_) | Command::Secret(_) => 
                prompt("your peer is disconnected. No messages sent"),
            _ => {}
        }
    }

    fn dial(&mut self, addr: SocketAddr, name: &str) {
        if let Err(e) = self.waiting_loop(addr, name) {
            prompt(&format!("connection was broken because: {}", e.descr));
        }
        prompt("you are in the menu now");
    }

    fn handle_incoming_connection(&self,
        mut connection: (TcpStream, SocketAddr),
        address: &SocketAddr,
        name: &str,
        ctx: &CryptoContext
    ) -> Result<bool, Error> {
        let msg = recieve(&mut connection.0)?;
        debug_prompt(&format!("I recieved [{:?}]", msg));
        if connection.1.ip() == address.ip() && msg.port == address.port() {
            match msg.t {
                // something ugly, pls help
                Type::Close => {
                    if let Some(data) = msg.data {
                        let raw_nonce = bincode::DefaultOptions::new()
                            .with_little_endian()
                            .deserialize::<u64>(&data);
                        if let Ok(nonce) = raw_nonce {
                            if nonce == ctx.nonce {
                                prompt("your peer disconnected. Wait for them or leave");
                                return Ok(true)
                            }
                        }
                    }
                },
                Type::SpeakPlain => {
                    if let Some(data) = msg.data {
                        let text = String::from_utf8(data)
                            .unwrap_or("<invalid encoding>".to_owned());
                        named_prompt(name, &text);
                    } else {
                        named_prompt(name, "<empty message>");
                    }
                }
                _ => {},
            }
        } else {
            if msg.t == Type::Request {
                send(&mut connection.0, Message::new_deny(self.cfg.port)).unwrap();
            }
        }
        Ok(false)
    }
}
