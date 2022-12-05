use std::net::{TcpListener, TcpStream, SocketAddr};
use std::os::unix::prelude::{AsFd, AsRawFd};
use std::io::stdin;

use nix::libc::STDIN_FILENO;
use nix::poll::{PollFd, PollFlags, poll};
use nix::errno::Errno;

use crate::config::Config;
use crate::cli::{interpret, Command};
use crate::error::{Error, ErrCode};
use crate::proto::message::Message;
use crate::proto::{send, handshake_init, decline};
use super::{prompt, execute};

pub fn idle_loop(config: Config) -> Result<(), Error> {
    // initialize necessary resources
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr)
        .map_err(|e| Error::new(ErrCode::Fatal, e.to_string()))?;
    let listener_fd = listener.as_fd();
    let mut watches = vec![
        PollFd::new(STDIN_FILENO, PollFlags::POLLIN),
        PollFd::new(listener_fd.as_raw_fd(), PollFlags::POLLIN)
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
            match interpret(buffer.trim()) {
                Err(e) => {
                    prompt(&e.descr);
                }
                Ok(Command::Exit) => break,
                Ok(cmd) => execute(cmd)?
            }
            buffer.clear();
        }
        if watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // TCP connection recieved, decline it
            let connection = listener.accept()
                .map_err(|e| Error::new(ErrCode::Fatal, e.to_string()))?;
            prompt(&format!("incoming connection from {} - declining", connection.1));
            decline(connection.0);
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
    let mut stream = TcpStream::connect(&desired_addr)
        .map_err(|e| Error::new(ErrCode::Network, e.to_string()))?;
    if true == handshake_init(&mut stream)? {
        let res = connected_loop(config, watches, listener, desired_addr);
        // check how did we exit from connected_loop
        // we either return to the idle loop (the user typed "exit") or stay in the wait loop
        // (our remote peer closed the connection)
        res
    } else {
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
                // TODO: special interpreter for wait loop
                match interpret(buffer.trim()) {
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
                    .map_err(|e| Error::new(ErrCode::Fatal, e.to_string()))?;
                if connection.1 == desired_addr {
                    prompt(&format!("incoming connection from {} - accepting", connection.1));
                    send(&mut stream, Message::new_accept())?;
                    let res = connected_loop(config, watches, listener, desired_addr);
                    return res;
                } else {
                    prompt(&format!("incoming connection from {} - declining", connection.1));
                    decline(connection.0);
                }
            }
        }
        Ok(())
    }
}

fn connected_loop(
    config: &Config,
    mut watches: [PollFd; 2],
    listener: &TcpListener,
    address: SocketAddr,
) -> Result<(), Error> {
    let mut buf = String::new();
    loop {
        match poll(&mut watches, -1) {
            Ok(0) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => continue,
            Ok(val) => val,
            Err(e) => return Err(Error::new(ErrCode::Fatal, e.to_string())),
        };
        if watches[0].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // process stdio
        }
        if watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // decline incoming tcp request / handle incoming message
        }
    }
    
}
