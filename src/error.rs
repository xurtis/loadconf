use std::fmt::{self, Display, Formatter};
use std::{io, error};
use std::convert::From;

use toml;

/// Errors produced when loading configuration.
#[derive(Debug)]
pub enum Error {
    File(io::Error),
    Deserialize(toml::de::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::File(_) => "Error opening or reading file",
            Error::Deserialize(_) => "Error deserializing file",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::File(ref err) => Some(err),
            Error::Deserialize(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::File(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::Deserialize(err)
    }
}
