use crate::{buffer::Buffer, error::ASEError};

use super::{block_type::BlockType, ColorType, ColorValue};

/// A single color with an associated name.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorBlock {
    /// The name associated with the color
    pub name: String,
    /// The specific color value of the block.
    pub color: ColorValue,
    /// The type of color
    pub color_type: ColorType,
}

impl ColorBlock {
    /// Creates a new ColorBlock with the given name, color type and color.
    pub fn new(name: String, color: ColorValue, color_type: ColorType) -> Self {
        Self {
            name,
            color_type,
            color,
        }
    }

    /// Write the block to the given [`Buffer`]
    pub(crate) fn write(self, buf: &mut Buffer) {
        buf.write_u16(BlockType::ColorEntry as u16);
        buf.write_u32(self.calculate_length());
        //name length, +1 for null terminator
        buf.write_u16(self.name.len() as u16 + 1);
        buf.write_null_terminated_utf_16_str(&self.name);

        //write color
        buf.write_slice(self.color.get_type());
        self.color.write_values(buf);
        buf.write_u16(self.color_type as u16);
    }

    /// Calculate the length of an color block.
    ///
    /// The length is calculate the following way:
    /// name length (2) + name * 2 (UTF 16) + null terminator (2)
    /// + color type (4) + color value (1/3/4) + type (2)
    pub(super) fn calculate_length(&self) -> u32 {
        2 + self.name.len() as u32 * 2 + 2 + 4 + self.color.calculate_length() + 2
    }

    /// Parses a [`ColorBlock`] from bytes.
    ///
    /// # Errors
    /// This function will return an error if parsing fails.
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, ASEError> {
        let name_length = u16::from_be_bytes(bytes[0..2].try_into()?);
        //read name bytes, but stop before null byte
        let name_bytes: Vec<u16> = bytes[2..(name_length as usize * 2)]
            .chunks_exact(2)
            .map(|bytes| u16::from_be_bytes(bytes.try_into().unwrap()))
            .collect();
        let name = String::from_utf16(&name_bytes)?;

        let color_value_start = name_length as usize * 2 + 2;
        let color_value = ColorValue::try_from(&bytes[color_value_start..])?;

        let color_type_start = color_value_start + color_value.calculate_length() as usize + 4;
        let color_type = ColorType::try_from(&bytes[color_type_start + 1])?;

        Ok(Self::new(name, color_value, color_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_calculates_length_correctly() {
        let block = ColorBlock::new("name".to_owned(), ColorValue::Gray(0.5), ColorType::Normal);
        assert_eq!(block.calculate_length(), 22);
    }

    #[test]
    fn it_writes_bytes_correctly() {
        let block = ColorBlock::new("name".to_owned(), ColorValue::Gray(0.5), ColorType::Normal);
        let mut buf = Buffer::with_capacity(22);
        block.write(&mut buf);
        assert_eq!(
            buf.into_vec(),
            vec![
                0, 1, 0, 0, 0, 22, 0, 5, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 71, 114, 97, 121, 63,
                0, 0, 0, 0, 2
            ]
        );
    }

    #[test]
    fn it_reads_bytes_correctly() {
        let block = ColorBlock::new("name".to_owned(), ColorValue::Gray(0.5), ColorType::Normal);
        assert_eq!(
            block,
            ColorBlock::parse(&[0, 5, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2])
            .unwrap()
        );
    }
}
