use std::net::{TcpListener, TcpStream, SocketAddr};
use std::os::unix::prelude::{AsFd, AsRawFd};
use std::io::{stdin, stdout, Write};

use nix::libc::STDIN_FILENO;
use nix::poll::{PollFd, PollFlags, poll};
use nix::errno::Errno;

use crate::config::Config;
use crate::cli::{interpret, Command};
use crate::error::{Error, ErrCode};
use crate::core::proto::{Message, Type};

use super::proto_callbacks::send;
use super::proto_callbacks::handshake_init;

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
    watches: &mut Vec<PollFd>,
    listener: &TcpListener,
    target_addr: SocketAddr,
) -> Result<(), Error> {
    let mut stream = TcpStream::connect(&target_addr)
        .map_err(|e| Error::new(ErrCode::Network, e.to_string()))?;
    if true == handshake_init(&mut stream)? {
        let stream_fd = stream.as_fd().as_raw_fd();
        watches.push(PollFd::new(stream_fd, PollFlags::POLLIN));
        let res = connected_loop(config, watches, listener, (&mut stream, target_addr));
        watches.pop();
        res
    } else {
        let mut buffer = String::new();
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
                if connection.1 == target_addr {
                    prompt(&format!("incoming connection from {} - accepting", connection.1));
                    send(&mut stream, Message::new_accept())?;
                    let stream_fd = stream.as_fd().as_raw_fd();
                    watches.push(PollFd::new(stream_fd, PollFlags::POLLIN));
                    let res = connected_loop(config, watches, listener, (&mut  stream, target_addr));
                    watches.pop();
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
    watches: &mut Vec<PollFd>,
    listener: &TcpListener,
    connection: (&mut TcpStream, SocketAddr),
) -> Result<(), Error> {
    if watches.len() != 3 {
        return Err(Error::new(ErrCode::Fatal, "Watches buffer is not balanced".to_owned()));
    }
    let mut buf = String::new();
    loop {
        match poll(watches.as_mut_slice(), -1) {
            Ok(0) | Err(Errno::EAGAIN) | Err(Errno::EINTR) => continue,
            Ok(val) => val,
            Err(e) => return Err(Error::new(ErrCode::Fatal, e.to_string())),
        };
        if watches[0].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // process stdio
        }
        if watches[1].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // decline incoming tcp
        }
        if watches[2].revents().unwrap_or(PollFlags::empty()).contains(PollFlags::POLLIN) {
            // read tcp connection
        }
    }
    
}

fn prompt(str: &str) {
    print!("\r<simi>: {}\n[you]: ", str);
    stdout().flush().unwrap();
}

fn execute(cmd: Command) -> Result<(), Error> {
    match cmd {
        Command::List => prompt("\"List\" is not implemented"),
        Command::Add(alias, socket) => 
            prompt(&format!("\"Add\" {} = {} is not implemented", alias, socket)),
        Command::DialIp(socket) => 
            prompt(&format!("\"DialIP\" {} is not implemented", socket)),
        Command::DialAlias(alias) => 
            prompt(&format!("\"DialAlias\" {} is not implemented", alias)),
        Command::Remove(alias) =>
            prompt(&format!("\"Remove\" {} is not implemented", alias)),
        Command::Secret(path) =>
            prompt(&format!("\"Secret\" path={:?} is not implemented", path)),
        Command::Exit => return Ok(())
    }
    Ok(())
}

// TODO error handling
fn decline(mut stream: TcpStream) {
    if let Ok(msg) = Message::deserialize(&mut stream) {
        if msg.t == Type::Request {
            send(&mut stream, Message::new_deny()).unwrap();
        }
    }
}
