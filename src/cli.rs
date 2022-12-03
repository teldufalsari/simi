use std::str::Split;
use std::net::SocketAddr;

use crate::error::{ErrCode, Error};

#[derive(Debug)]
pub enum Command {
    Exit,
    List,
    Add(String, String),
    Remove(String),
    DialIp(String),
    DialAlias(String),
    Secret(Option<String>),
}

/// Takes *trimmed* string and tries to interpret it as a command with arguments
pub fn interpret(line: &str) -> Result<Command, Error> {
    let mut argv = line.split(" ");
    let cmd = argv.next();
    let args = argv; 
    match cmd {
        Some("list") => list(args),
        Some("exit") => exit(args),
        Some("add") => add(args),
        Some("remove") => remove(args),
        Some("dial") => dial(args),
        Some("secret") => secret(args),
        Some(cmd) => Err(Error::new(ErrCode::UnknownCommand, format!("unknown command \"{}\"", cmd))),
        None => Err(Error::new(ErrCode::EmptyLine, String::new()))
    }
}

fn list(mut args: Split<&str>) -> Result<Command, Error> {
    if args.next().is_some() {
        Err(Error::new(ErrCode::WrongArgs, "usage: list".to_owned()))
    } else {
        Ok(Command::List)
    }
}

fn exit(mut args: Split<&str>) -> Result<Command, Error> {
    if args.next().is_some() {
        Err(Error::new(ErrCode::WrongArgs, "usage: exit".to_owned()))
    } else {
        Ok(Command::Exit)
    }
}

fn add(args: Split<&str>) -> Result<Command, Error> {
    let args = args.collect::<Vec<_>>();
    if args.len() != 2 {
        return Err(Error::new(ErrCode::WrongArgs, "usage: add <alias> <ip:port>".to_owned()));
    }
    if args[1].parse::<SocketAddr>().is_err() {
        return Err(Error::new(ErrCode::WrongArgs, format!("\"{}\" is not a valid socket address", args[1])));
    }
    Ok(Command::Add(args[0].to_owned(), args[1].to_owned()))
}

fn remove(args: Split<&str>) -> Result<Command, Error> {
    let args = args.collect::<Vec<_>>();
    if args.len() != 1 {
        return Err(Error::new(ErrCode::WrongArgs, "usage: remove <alias>".to_owned()));
    }
    Ok(Command::Remove(args[0].to_owned()))
}

fn dial(args: Split<&str>) -> Result<Command, Error> {
    let args = args.collect::<Vec<_>>();
    if args.len() != 1 {
        Err(Error::new(ErrCode::WrongArgs, "usage: dial <alias> or dial <ip:port>".to_owned()))
    } else if args[0].parse::<SocketAddr>().is_ok() {
        Ok(Command::DialIp(args[0].to_owned()))
    } else {
        Ok(Command::DialAlias(args[0].to_owned()))
    }
}

// Temporary function implementation
// Use clap crate for multiple arguments, if there are going to be any
fn secret(args: Split<&str>) -> Result<Command, Error> {
    let args = args.collect::<Vec<_>>();
    if args.len() > 1 {
        return Err(Error::new(ErrCode::WrongArgs, "usage: --secret [--path=/path/to/file]".to_owned()));
    }
    if args.len() == 0 {
        return Ok(Command::Secret(None));
    }
    let mut arg = args[0].split("=");
    let key = arg.next().unwrap();
    if key != "--path" {
        return Err(Error::new(ErrCode::WrongArgs, format!("unknown argument \"{}\"", key)));
    }
    let path = match arg.next() {
        None => {return Err(Error::new(ErrCode::WrongArgs, "usage: --path=/path/to/file.png".to_owned()));}
        Some(val) => val.to_owned()
    };
    Ok(Command::Secret(Some(path)))
}
