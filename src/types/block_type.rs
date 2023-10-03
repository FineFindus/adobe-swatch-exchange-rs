use crate::error::ASEError;

/// Block identifier
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BlockType {
    /// Indicates the start of a [`Group`]
    GroupStart = 0xc001,
    /// Indicates the end of a [`Group`]
    GroupEnd = 0xc002,
    /// Indicates the start of a [`ColorBlock`]
    ColorEntry = 0x0001,
}

impl TryFrom<u16> for BlockType {
    type Error = ASEError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(Self::ColorEntry),
            0xc001 => Ok(Self::GroupStart),
            0xc002 => Ok(Self::GroupEnd),
            _ => Err(ASEError::Invalid),
        }
    }
}
