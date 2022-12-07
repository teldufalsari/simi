use std::net::{TcpListener, TcpStream, SocketAddr};
use std::os::unix::prelude::{AsFd, AsRawFd};
use std::io::stdin;

use nix::libc::STDIN_FILENO;
use nix::poll::{PollFd, PollFlags, poll};
use nix::errno::Errno;

use crate::config::Config;
use crate::cli::{menu, dialogue, Command};
use crate::error::{Error, ErrCode, convert_err};
use crate::proto::message::{Type, Message};
use crate::proto::{handshake_init, decline, recieve, accept_or_decline, send};
use super::{prompt, empty_prompt, execute};

pub struct Application {
    cfg: Config,
    addr: SocketAddr,
    listener: TcpListener,
    watches: [PollFd; 2],
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
        Ok(Self { cfg, addr, listener, watches})
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let mut buffer = String::new();
        prompt("connected to the network");
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

    fn waiting_loop(&mut self, desired_addr: SocketAddr) -> Result<(), Error> {
        // First try connecting to the remote peer
        let mut stream = TcpStream::connect(&desired_addr)
            .map_err(|e| Error::new(ErrCode::Network, e.to_string()))?;
        if true == handshake_init(&mut stream, self.cfg.port)? {
            // On success - wait until this connection is closed
            let cause = self.connected_loop(desired_addr)?;
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
                    Err(e) => {
                        prompt(&e.descr);
                    }
                    Ok(Command::Exit) => break,
                    Ok(cmd) => execute(cmd)?
                }
                buffer.clear();
            }
            if self.watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
                // TCP connection recieved, decide on it
                let connection = self.listener.accept()
                    .map_err(|e| convert_err(e, ErrCode::Fatal))?;
                match accept_or_decline(connection, self.cfg.port, &desired_addr) {
                    Ok(true) => {
                        let cause = self.connected_loop(desired_addr)?;
                        if cause == CloseCaused::Locally {
                            return Ok(());
                        }
                    }
                    Ok(false) => continue,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    fn connected_loop(&mut self, address: SocketAddr) -> Result<CloseCaused, Error> {
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
                        send(&mut stream, Message::new_close(self.cfg.port))?;
                        return Ok(CloseCaused::Locally)
                    }
                    Ok(Command::SpeakPlain(text)) => {
                        let mut stream = TcpStream::connect(address)
                            .map_err(|e| convert_err(e, ErrCode::Network))?;
                        //prompt(&format!("Sending \"{}\" to {}", &text, address));
                        send(&mut stream, Message::new_speak_plain(self.cfg.port, text.into_bytes()))?;
                        empty_prompt();
                    }
                    Ok(cmd) => execute(cmd)?
                }
                buffer.clear();
            }
            if self.watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
                // decline incoming tcp request / handle incoming message
                let mut connection = self.listener.accept()
                    .map_err(|e| convert_err(e, ErrCode::Fatal))?;
                    // TODO: separate serialization errors from network errors
                    // Actualy, none of them is fatal; we should just return to the idle loop
                    let msg = recieve(&mut connection.0)?;
                    if connection.1.ip() == address.ip() && msg.port == address.port() {
                    //prompt(&format!("I recieved [{:?}]", msg));
                    match msg.t {
                        Type::Close => {
                            prompt("your peer disconnected. Wait for them or leave");
                            return Ok(CloseCaused::ByRemote)
                        },
                        // TODO: display the peer name instead of system
                        Type::SpeakPlain => {
                            if let Some(data) = msg.data {
                                let text = String::from_utf8(data)
                                    .unwrap_or("<invalid encoding>".to_owned());
                                prompt(&text);
                            } else {
                                prompt("empty message recieved")
                            }
                        }
                        _ => continue,
                    }
                } else {
                    decline(connection.0, self.cfg.port);
                }
            }
        }
    }

    fn menu_execute(&mut self, cmd: Command) {
        match cmd {
            Command::List => {
                if self.cfg.contacts.is_empty() {
                    prompt("empty");
                } else {
                    for contact in self.cfg.contacts.iter() {
                        println!("{} : {}", contact.0, contact.1);
                    }
                    empty_prompt();
                }
            }
            Command::Add(alias, addr) => {
                self.cfg.contacts.insert(alias, addr);
                empty_prompt();
            }
            Command::Remove(alias) => {
                if let None = self.cfg.contacts.remove(&alias) {
                    prompt(&format!("alias {} not found", alias));
                } else {
                    empty_prompt();
                }
            }
            Command::Save => {
                if let Err(e) = self.cfg.save() {
                    prompt(&format!("cannot save config: {}", e));
                } else {
                    empty_prompt();
                }
            }
            Command::DialIp(ip) => {
                let addr = ip.parse::<SocketAddr>().unwrap();
                self.dial(addr)
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
                self.dial(addr)
            }
            _ => {}
        }
    }

    fn dial(&mut self, addr: SocketAddr) {
        if let Err(e) = self.waiting_loop(addr) {
            prompt(&format!("connection was broken because: {}", e.descr));
        }
        prompt("you are in the menu now");
    }
}
