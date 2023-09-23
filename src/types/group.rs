use crate::buffer::Buffer;

use super::{block_type::BlockType, ColorBlock};

///Represents a named collection of colors
#[derive(Debug, Clone)]
pub struct Group<'a> {
    /// The name of the group
    pub name: &'a str,
    /// The colors in the group
    pub blocks: Vec<ColorBlock<'a>>,
}

impl<'a> Group<'a> {
    /// Creates a new group of colors, grouped together with the specified name.
    pub fn new(name: &'a str, blocks: Vec<ColorBlock<'a>>) -> Self {
        Self { name, blocks }
    }

    /// Write the group to the given [`Buffer`]
    pub(crate) fn write(self, buf: &mut Buffer) {
        buf.write_u32(BlockType::GroupStart as u32);
        buf.write_u32(self.calculate_length());

        //name length, +1 for null terminator
        buf.write_u16(self.name.len() as u16 + 1);
        buf.write_null_terminated_utf_16_str(self.name);

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
