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

pub fn idle_loop(config: Config) -> Result<(), Error> {
    // create TCP listener
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr)
        .map_err(|e| convert_err(e, ErrCode::Fatal))?;

    // allocate necessary resources
    let listener_fd = listener.as_fd().as_raw_fd();
    let mut watches = [
        PollFd::new(STDIN_FILENO, PollFlags::POLLIN),
        PollFd::new(listener_fd, PollFlags::POLLIN)
    ];
    let mut buffer = String::new();

    prompt("connected to the network");
    loop {
        match poll(watches.as_mut_slice(), -1) {
            Ok(0) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => continue,
            Ok(val) => val,
            Err(e) => return Err(Error::new(ErrCode::Fatal, e.to_string())),
        };
        if watches[0].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // Events in stdio
            // read line, interpret, execute
            stdin().read_line(&mut buffer).unwrap();
            match menu::interpret(buffer.trim()) {
                Err(e) => prompt(&e.descr),
                Ok(Command::Exit) => break,
                Ok(Command::DialIp(addr)) => {
                    let desired_addr = addr.parse::<SocketAddr>().unwrap();
                    if let Err(e) = waiting_loop(&config, watches, &listener, desired_addr) {
                        prompt(&format!("connection was broken because: {}", e.descr));
                    }
                    prompt("you are in the menu now");
                }
                Ok(cmd) => execute(cmd)?
            }
            buffer.clear();
        }
        if watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // TCP connection recieved, decline it
            let connection = listener.accept()
                .map_err(|e| Error::new(ErrCode::Fatal, e.to_string()))?;
            prompt(&format!("incoming connection from {} - declining", connection.1));
            decline(connection.0, config.port);
        }
    }
    Ok(())
}

fn waiting_loop(
    config: &Config,
    mut watches: [PollFd; 2],
    listener: &TcpListener,
    desired_addr: SocketAddr,
) -> Result<(), Error> {
    // First try connecting to the remote peer
    let mut stream = TcpStream::connect(&desired_addr)
        .map_err(|e| Error::new(ErrCode::Network, e.to_string()))?;
    if true == handshake_init(&mut stream, config.port)? {
        // On success - wait until this connection is closed
        let cause = connected_loop(config, watches, listener, desired_addr)?;
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
        match poll(&mut watches, -1) {
            Ok(0) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => continue,
            Ok(val) => val,
            Err(e) => return Err(Error::new(ErrCode::Fatal, e.to_string())),
        };
        if watches[0].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
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
        if watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // TCP connection recieved, decide on it
            let connection = listener.accept()
                .map_err(|e| convert_err(e, ErrCode::Fatal))?;
            match accept_or_decline(connection, config.port, &desired_addr) {
                Ok(true) => {
                    let cause = connected_loop(config, watches, listener, desired_addr)?;
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

#[derive(PartialEq)]
enum CloseCaused {
    ByRemote,
    Locally,
}

fn connected_loop(
    config: &Config,
    mut watches: [PollFd; 2],
    listener: &TcpListener,
    address: SocketAddr,
) -> Result<CloseCaused, Error> {
    prompt("connected to the peer");
    let mut buffer = String::new();
    loop {
        match poll(&mut watches, -1) {
            Ok(0) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => continue,
            Ok(val) => val,
            Err(e) => return Err(Error::new(ErrCode::Fatal, e.to_string())),
        };
        if watches[0].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            stdin().read_line(&mut buffer).unwrap();
            match dialogue::interpret(buffer.trim()) {
                Err(e) => {
                    prompt(&e.descr);
                }
                Ok(Command::Exit) => {
                    let mut stream = TcpStream::connect(address)
                        .map_err(|e| convert_err(e, ErrCode::Network))?;
                    send(&mut stream, Message::new_close(config.port))?;
                    return Ok(CloseCaused::Locally)
                }
                Ok(Command::SpeakPlain(text)) => {
                    let mut stream = TcpStream::connect(address)
                        .map_err(|e| convert_err(e, ErrCode::Network))?;
                    //prompt(&format!("Sending \"{}\" to {}", &text, address));
                    send(&mut stream, Message::new_speak_plain(config.port, text.into_bytes()))?;
                    empty_prompt();
                }
                Ok(cmd) => execute(cmd)?
            }
            buffer.clear();
        }
        if watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // decline incoming tcp request / handle incoming message
            let mut connection = listener.accept()
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
                decline(connection.0, config.port);
            }
        }
    }
}
