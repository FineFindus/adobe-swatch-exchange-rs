use crate::buffer::Buffer;

use super::{block_type::BlockType, ColorType, ColorValue};

/// A single color with an associated name.
#[derive(Debug, Clone)]
pub struct ColorBlock<'a> {
    /// The name associated with the color
    pub name: &'a str,
    /// The specific color value of the block.
    pub color: ColorValue,
    /// The type of color
    pub color_type: ColorType,
}

impl<'a> ColorBlock<'a> {
    /// Creates a new ColorBlock with the given name, color type and color.
    pub fn new(name: &'a str, color: ColorValue, color_type: ColorType) -> Self {
        Self {
            name,
            color_type,
            color,
        }
    }

    /// Write the block to the given [`Buffer`]
    pub(crate) fn write(self, buf: &mut Buffer) {
        buf.write_u32(BlockType::ColorEntry as u32);
        buf.write_u32(self.calculate_length());
        //name length, +1 for null terminator
        buf.write_u16(self.name.len() as u16 + 1);
        buf.write_null_terminated_utf_16_str(self.name);

        //write color
        buf.write_slice(self.color.get_type());
        self.color.write_values(buf);
        buf.write_u16(self.color_type as u16);
    }

    /// Calculate the length of an color block.
    ///
    /// The length is calculate the following way:
    /// name length (2) + name + null terminator (2)
    /// + color type (4) + color value (1/3/4) + type (2)
    pub(super) fn calculate_length(&self) -> u32 {
        2 + self.name.len() as u32 + 2 + 4 + self.color.calculate_length() + 2
    }
}
