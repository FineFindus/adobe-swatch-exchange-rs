use crate::buffer::Buffer;

use super::{block_type::BlockType, ColorBlock};

///Represents a named collection of colors
#[derive(Debug, Clone)]
pub struct Group {
    /// The name of the group
    pub name: String,
    /// The colors in the group
    pub blocks: Vec<ColorBlock>,
}

impl Group {
    /// Creates a new group of colors, grouped together with the specified name.
    pub fn new(name: String, blocks: Vec<ColorBlock>) -> Self {
        Self { name, blocks }
    }

    /// Write the group to the given [`Buffer`]
    pub(crate) fn write(self, buf: &mut Buffer) {
        buf.write_u32(BlockType::GroupStart as u32);
        buf.write_u32(self.calculate_length());

        //name length, +1 for null terminator
        buf.write_u16(self.name.len() as u16 + 1);
        buf.write_null_terminated_utf_16_str(&self.name);

        //write colors
        self.blocks.into_iter().for_each(|block| block.write(buf));

        buf.write_u32(BlockType::GroupEnd as u32);
    }

    /// Calculate the length of an group.
    ///
    /// The length is calculate the following way:
    /// name length (2) + name + null terminator (2)
    /// + color entry type (2) + color entry length
    pub(super) fn calculate_length(&self) -> u32 {
        2 + self.name.len() as u32
            + 2
            + self
                .blocks
                .iter()
                .map(|block| block.calculate_length() + 2)
                .sum::<u32>()
    }
}

#[cfg(test)]
mod tests {
    use crate::{ColorType, ColorValue};

    use super::*;

    #[test]
    fn it_calculates_length_correctly() {
        let group = Group::new(
            "group name".to_owned(),
            vec![
                ColorBlock::new(
                    "light grey".to_owned(),
                    ColorValue::Gray(0.5),
                    ColorType::Normal,
                ),
                ColorBlock::new(
                    "dark red".to_owned(),
                    ColorValue::Rgb(0.5, 0.3, 0.1),
                    ColorType::Normal,
                ),
            ],
        );
        assert_eq!(group.calculate_length(), 72);
    }

    #[test]
    fn it_writes_bytes_correctly() {
        let group = Group::new(
            "group name".to_owned(),
            vec![
                ColorBlock::new(
                    "light grey".to_owned(),
                    ColorValue::Gray(0.5),
                    ColorType::Normal,
                ),
                ColorBlock::new(
                    "dark red".to_owned(),
                    ColorValue::Rgb(0.5, 0.3, 0.1),
                    ColorType::Normal,
                ),
            ],
        );
        let mut buf = Buffer::with_capacity(72);
        group.write(&mut buf);
        assert_eq!(
            buf.into_vec(),
            vec![
                0, 0, 192, 1, 0, 0, 0, 72, 0, 11, 0, 103, 0, 114, 0, 111, 0, 117, 0, 112, 0, 32, 0,
                110, 0, 97, 0, 109, 0, 101, 0, 0, 0, 0, 0, 1, 0, 0, 0, 24, 0, 11, 0, 108, 0, 105,
                0, 103, 0, 104, 0, 116, 0, 32, 0, 103, 0, 114, 0, 101, 0, 121, 0, 0, 71, 114, 97,
                121, 63, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 30, 0, 9, 0, 100, 0, 97, 0, 114, 0,
                107, 0, 32, 0, 114, 0, 101, 0, 100, 0, 0, 82, 71, 66, 32, 63, 0, 0, 0, 62, 153,
                153, 154, 61, 204, 204, 205, 0, 2, 0, 0, 192, 2
            ]
        );
    }
}
