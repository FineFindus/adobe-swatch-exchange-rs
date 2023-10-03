use std::{array, fmt::Display, io, string};

/// Indicates a failure in decoding the ASE.
#[derive(Debug)]
pub enum ASEError {
    /// An error occurred while reading data from the provided source.
    Io(io::Error),
    /// An error was encountered while parsing the ASE.
    ///
    /// This means that the input data did not conform to the ASE specification.
    Invalid,
}

impl Display for ASEError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASEError::Io(err) => err.fmt(f),
            ASEError::Invalid => write!(f, "ASE file is invalid"),
        }
    }
}

impl std::error::Error for ASEError {}

impl From<io::Error> for ASEError {
    fn from(value: io::Error) -> Self {
        ASEError::Io(value)
    }
}

impl From<array::TryFromSliceError> for ASEError {
    fn from(_value: array::TryFromSliceError) -> Self {
        ASEError::Invalid
    }
}

impl From<string::FromUtf16Error> for ASEError {
    fn from(_value: string::FromUtf16Error) -> Self {
        ASEError::Invalid
    }
}
