use std::str::Split;
use std::net::SocketAddr;

use crate::error::{ErrCode, Error};
use super::Command;

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
        Some("save") => save(args),
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

fn save(mut args: Split<&str>) -> Result<Command, Error> {
    if args.next().is_some() {
        Err(Error::new(ErrCode::WrongArgs, "usage: save".to_owned()))
    } else {
        Ok(Command::Save)
    }
}
