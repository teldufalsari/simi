use std::net::{TcpListener, TcpStream, SocketAddr};
use std::os::unix::prelude::{AsFd, AsRawFd};
use std::io::{stdin, stdout, Write};

use nix::libc::STDIN_FILENO;
use nix::poll::{PollFd, PollFlags, poll};
use nix::errno::Errno;

use crate::config::Config;
use crate::cli::{interpret, Command};
use crate::error::{Error, ErrCode};

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
            let connection = listener.accept().expect("French inquisition");
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
) {
 // implement me
}

fn connected_loop(
    config: &Config,
    watches: &mut Vec<PollFd>,
    listener: &TcpListener,
    connection: (TcpStream, SocketAddr),
) -> Result<(), Error> {
    if watches.len() != 3 {
        return Err(Error::new(ErrCode::Fatal, "Watches buffer is not balanced".to_owned()));
    }
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
    print!("<simi>: {}\n[you]: ", str);
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

fn decline(stream: TcpStream) {
    // implement me
}