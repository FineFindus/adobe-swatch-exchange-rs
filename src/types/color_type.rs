use crate::error::ASEError;

/// Type of Color in the ASE file.
/// Specifies how the color behaves in a document.
///
/// Information from <https://pypi.org/project/swatch/>
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ColorType {
    /// Represents Global colors in ASE files.
    ///
    /// Essentially the same as Process colors, but with the property that changes to them reflect
    /// throughout the entire artwork. This makes them akin to "color references", useful for tasks
    /// such as reskinning an existing document.
    Global = 0,

    /// Represents Spot colors in ASE files.
    ///
    /// Spot colors have the unique characteristic that they allow the creation of new swatches
    /// based on "tints" or, essentially, some screened value of that color. They are inherently global.
    /// However, tints, despite being part of a file, cannot be stored/exchanged as swatches.
    /// Even tools like Illustrator won't save them, which is likely due to the file format's constraints.
    Spot = 1,

    /// Represents Process colors in ASE files.
    ///
    /// Process colors are standard colors, typically the default when defining a new color in tools like Illustrator.
    /// They are mixed from either RGB or CMYK, depending on the document's color mode.
    #[default]
    Normal = 2,
}

impl TryFrom<&u8> for ColorType {
    type Error = ASEError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ColorType::Global),
            1 => Ok(ColorType::Spot),
            2 => Ok(ColorType::Normal),
            _ => Err(ASEError::ColorTypeError),
        }
    }
}
