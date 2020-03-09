use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Generic(String),
    Utf8Error(std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Generic(ref err) => write!(f, "Generic error: {}", err),
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Utf8Error(ref err) => write!(f, "UTF8 error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Generic(ref err) => err,
            Error::Io(ref err) => err.description(),
            Error::Utf8Error(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Error::Generic(ref _err) => None,
            Error::Io(ref err) => Some(err),
            Error::Utf8Error(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Generic(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

pub fn make_generic(msg: &str) -> Error {
    Error::Generic(msg.to_string())
}
