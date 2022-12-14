#[derive(Debug)]
/// General application error type
pub struct Error {
    code: ErrCode,
    pub descr: String,
}

#[derive(Debug)]
pub enum ErrCode {
    /// This is not an error
    Ok,

    /// Empty line has been entered.
    /// An application should skip this line and print
    /// command line prompt again.
    EmptyLine,

    /// Wrong arguments submitted with an existing command.
    /// An application should print error message to the user and
    /// ignore this command
    WrongArgs,

    /// An unknown command has been entered.
    /// An application should skip this command.
    UnknownCommand,

    /// Some network problems
    /// 
    /// Not necessary fatal
    Network,

    /// Serialization error
    Serial,

    Filesys,

    /// A fatal, unrecoverable error occured.
    /// An application should print error message
    /// and exit
    Fatal,
}

impl Error {
    /// Creates new `Error` instance with given code and description
    pub fn new(code: ErrCode, descr: String) -> Error {
        Error{code, descr}
    }
}

/// Converts any other error to the internal error type
pub fn convert_err<E: ToString>(e: E, code: ErrCode) -> Error {
    Error::new(code, e.to_string())
}
