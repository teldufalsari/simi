use std::str::Split;

use crate::error::{ErrCode, Error};
use super::Command;


pub fn interpret(line: &str) -> Result<Command, Error> {
    if line.starts_with("--") {
        let mut argv = line[2..].split(" ");
        let cmd = argv.next();
        let args = argv;
        match cmd {
            Some("exit") => exit(args),
            Some("secret") => secret(args),
            Some(cmd) => 
                Err(Error::new(ErrCode::UnknownCommand, format!("unknown command \"{}\"", cmd))),
            None => Err(Error::new(ErrCode::EmptyLine, String::new())),
        }
    } else {
        Ok(Command::SpeakPlain(line.to_owned()))
    }
}

fn exit(mut args: Split<&str>) -> Result<Command, Error> {
    if args.next().is_some() {
        Err(Error::new(ErrCode::WrongArgs, "usage: exit".to_owned()))
    } else {
        Ok(Command::Exit)
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
