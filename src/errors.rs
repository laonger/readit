use std::fmt;
use std::result::Result as StdResult;
use std::io::Error as IoError;
use lancedb::error::Error as LanceError;
use async_openai::error::OpenAIError;
use arrow::error::ArrowError;


#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    LanceError(LanceError),
    OpenAIError(OpenAIError),
    ArrowError(ArrowError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "IoError: {}", err),
            Error::LanceError(err) => write!(f, "LanceError: {}", err),
            Error::OpenAIError(err) => write!(f, "OpenAIError: {}", err),
            Error::ArrowError(err) => write!(f, "ArrowError: {}", err),
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::IoError(err)
    }
}
impl From<LanceError> for Error {
    fn from(err: LanceError) -> Self {
        Error::LanceError(err)
    }
}
impl From<OpenAIError> for Error {
    fn from(err: OpenAIError) -> Self {
        Error::OpenAIError(err)
    }
}
impl From<ArrowError> for Error {
    fn from(err: ArrowError) -> Self {
        Error::ArrowError(err)
    }
}


pub type Result<T> = StdResult<T, Error>;
