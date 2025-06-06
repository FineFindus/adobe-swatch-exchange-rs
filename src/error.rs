use std::{array, fmt::Display, io, string};

/// Indicates a failure in decoding the ASE.
#[derive(Debug)]
pub enum ASEError {
    /// An error occurred while reading data from the provided source.
    Io(io::Error),
    /// An error was encountered while parsing the ASE.
    ///
    /// This means that the input data did not conform to the ASE specification.
    Invalid(ConformationError),
    /// An error occurred due to an invalid color format.
    ///
    /// Valid color formats are: [`CMYK`](crate::types::ColorValue::Cmyk), [`RGB`](crate::types::ColorValue::Rgb),
    /// [`Lab`](crate::types::ColorValue::Lab) and [`Gray`](crate::types::ColorValue::Gray)
    ColorFormat,
    /// An error occurred due to UTF-16 parsing issues.
    UTF16Error,
    /// An error occurred due to an invalid [`ColorType`](crate::types::ColorType).
    ColorTypeError,
    /// An error occurred due to an invalid block type.
    BlockTypeError,
    /// An error occurred while parsing the input data.
    InputDataParseError,
}

/// Indicates the cause of the file being an invalid ASE.
#[derive(Debug)]
pub enum ConformationError {
    /// An invalid file version was detected.
    ///
    /// Currently only version `1.0` of the format exists.
    FileVersion,
    /// An invalid file signature was detected.
    ///
    /// The file signature must be `ASEF`.
    FileSignature,
    /// Groups must be terminated with a `GroupEnd` block.
    GroupEnd,
}

impl Display for ASEError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASEError::Io(err) => err.fmt(f),
            ASEError::Invalid(err) => write!(f, "ASE file is invalid: {err}"),
            ASEError::ColorFormat => write!(f, "Error parsing color format"),
            ASEError::UTF16Error => write!(f, "Error converting UTF16"),
            ASEError::ColorTypeError => write!(f, "Error converting ColorType"),
            ASEError::BlockTypeError => write!(f, "Error converting BlockType"),
            ASEError::InputDataParseError => write!(f, "Error parsing input data"),
        }
    }
}

impl Display for ConformationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConformationError::FileVersion => write!(f, "File version is not supported"),
            ConformationError::FileSignature => write!(f, "Invalid file signature found"),
            ConformationError::GroupEnd => write!(f, "Blocks must end to be valid"),
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
        ASEError::InputDataParseError
    }
}

impl From<string::FromUtf16Error> for ASEError {
    fn from(_value: string::FromUtf16Error) -> Self {
        ASEError::UTF16Error
    }
}
