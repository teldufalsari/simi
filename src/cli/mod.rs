pub mod menu;
pub mod dialogue;

#[derive(Debug)]
pub enum Command {
    Exit,
    List,
    Add(String, String),
    Remove(String),
    DialIp(String),
    DialAlias(String),
    Secret(Option<String>),
    SpeakPlain(String),
}
