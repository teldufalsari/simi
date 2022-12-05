use std::net::{TcpListener, TcpStream, SocketAddr};
use std::os::unix::prelude::{AsFd, AsRawFd};
use std::io::stdin;

use nix::libc::STDIN_FILENO;
use nix::poll::{PollFd, PollFlags, poll};
use nix::errno::Errno;

use crate::config::Config;
use crate::cli::{menu, dialogue, Command};
use crate::error::{Error, ErrCode, convert_err};
use crate::proto::message::Message;
use crate::proto::{send, handshake_init, decline};
use super::{prompt, execute};

pub fn idle_loop(config: Config) -> Result<(), Error> {
    // create TCP listener
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr)
        .map_err(|e| convert_err(e, ErrCode::Fatal))?;

    // allocate necessary resources
    let listener_fd = listener.as_fd().as_raw_fd();
    let mut watches = vec![
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
    // First try connecting to the remote peer
    let mut stream = TcpStream::connect(&desired_addr)
        .map_err(|e| Error::new(ErrCode::Network, e.to_string()))?;
    if true == handshake_init(&mut stream)? {
        // On success - wait until this connection is closed
        let cause = connected_loop(config, watches, listener, desired_addr)?;
        if cause == CloseCaused::Locally {
            // if it was closed by the local user - return to the idle loop,
            // otherwise go to the wait loop
            return Ok(());
        }
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
            // TODO: special interpreter for wait loop
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
            if connection.1 == desired_addr {
                prompt(&format!("incoming connection from {} - accepting", connection.1));
                send(&mut stream, Message::new_accept())?;
                let cause = connected_loop(config, watches, listener, desired_addr)?;
                if cause == CloseCaused::Locally {
                    return Ok(());
                }
            } else {
                prompt(&format!("incoming connection from {} - declining", connection.1));
                decline(connection.0);
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
            let connection = listener.accept()
                .map_err(|e| convert_err(e, ErrCode::Fatal))?;

            if connection.1 == address {
                // print message or return
            } else {
                decline(connection.0)
            }
        }
    }
    
}
