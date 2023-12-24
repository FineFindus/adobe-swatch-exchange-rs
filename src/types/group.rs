use crate::{buffer::Buffer, error::ASEError};

use super::{block_type::BlockType, ColorBlock};

/// Represents a named collection of colors
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Group {
    /// The name of the group
    pub name: String,
    /// The colors in the group
    pub blocks: Vec<ColorBlock>,
}

/// An type to handle processing of files during parsing.
///
/// This is a workaround for groups with size values that
/// do not include all color blocks.
#[derive(Debug, PartialEq)]
pub(crate) enum GroupHold {
    /// Colors are being collected into a found parent group.
    HoldingBuilding,
    /// Colors were already collected by the Group::parse() function.
    HoldingBuilt,
    /// Colors are currently being collected in the global context.
    Empty,
}

impl Group {
    /// Creates a new group of colors, grouped together with the specified name.
    pub fn new(name: String, blocks: Vec<ColorBlock>) -> Self {
        Self { name, blocks }
    }

    /// Write the group to the given [`Buffer`]
    pub(crate) fn write(self, buf: &mut Buffer) {
        buf.write_u16(BlockType::GroupStart as u16);
        buf.write_u32(self.calculate_length());

        //name length, +1 for null terminator
        buf.write_u16(self.name.len() as u16 + 1);
        buf.write_null_terminated_utf_16_str(&self.name);

        //write colors
        self.blocks.into_iter().for_each(|block| block.write(buf));

        buf.write_u16(BlockType::GroupEnd as u16);
    }

    /// Calculate the length of an group.
    ///
    /// The length is calculate the following way:
    /// name length (2) + name (* 2, UTF 16) + null terminator (2)
    /// + color entry type (2) + color entry length
    pub(super) fn calculate_length(&self) -> u32 {
        2 + self.name.len() as u32 * 2
            + 2
            + self
                .blocks
                .iter()
                .map(|block| block.calculate_length() + 2 + 4)
                .sum::<u32>()
    }

    /// Parses a [`Group`] from bytes.
    ///
    /// This will extract the name and than try to parse the left-over bytes
    /// as [`ColorBlock`]s. It stops when either the given bytes are 'empty',parsing a [`ColorBlock`]
    /// fails or the next block is not a [`ColorBlock`].
    ///
    /// # Errors
    /// This function will return an error if either the name cannot be constructed, or
    /// if it cannot be correctly parsed. In either case an [`ASEError::Invalid`] is returned.
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, ASEError> {
        let name_length = u16::from_be_bytes(
            bytes
                .get(0..2)
                .ok_or(ASEError::InputDataParseError)?
                .try_into()?,
        );
        //read name bytes, but stop before not byte
        let name_bytes: Vec<u16> = bytes
            .get(2..(name_length as usize * 2))
            .ok_or(ASEError::InputDataParseError)?
            .chunks_exact(2)
            .map(|bytes| u16::from_be_bytes(bytes.try_into().unwrap()))
            .collect();
        let name = String::from_utf16(&name_bytes)?;

        let mut pointer = name_length as usize * 2 + 2;
        let mut blocks = Vec::new();
        loop {
            if pointer >= bytes.len() - 1 {
                break;
            }

            let block_type = BlockType::try_from(u16::from_be_bytes(
                bytes
                    .get(pointer..(pointer + 2))
                    .ok_or(ASEError::InputDataParseError)?
                    .try_into()?,
            ))?;

            if block_type != BlockType::ColorEntry {
                break;
            }
            pointer += 2;

            let block_length = u32::from_be_bytes(
                bytes
                    .get(pointer..(pointer + 4))
                    .ok_or(ASEError::InputDataParseError)?
                    .try_into()?,
            ) as usize;
            pointer += 4;

            let Ok(block) =
                ColorBlock::parse(bytes.get(pointer..).ok_or(ASEError::InputDataParseError)?)
            else {
                break;
            };
            pointer += block_length;
            blocks.push(block);
        }

        Ok(Self::new(name, blocks))
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
        assert_eq!(group.calculate_length(), 108);
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
        let mut buf = Buffer::with_capacity(108);
        group.write(&mut buf);
        assert_eq!(
            buf.into_vec(),
            vec![
                192, 1, 0, 0, 0, 108, 0, 11, 0, 103, 0, 114, 0, 111, 0, 117, 0, 112, 0, 32, 0, 110,
                0, 97, 0, 109, 0, 101, 0, 0, 0, 1, 0, 0, 0, 34, 0, 11, 0, 108, 0, 105, 0, 103, 0,
                104, 0, 116, 0, 32, 0, 103, 0, 114, 0, 101, 0, 121, 0, 0, 71, 114, 97, 121, 63, 0,
                0, 0, 0, 2, 0, 1, 0, 0, 0, 38, 0, 9, 0, 100, 0, 97, 0, 114, 0, 107, 0, 32, 0, 114,
                0, 101, 0, 100, 0, 0, 82, 71, 66, 32, 63, 0, 0, 0, 62, 153, 153, 154, 61, 204, 204,
                205, 0, 2, 192, 2
            ]
        );
    }

    #[test]
    fn it_reads_bytes_correctly() {
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
        assert_eq!(
            group,
            Group::parse(&[
                0, 11, 0, 103, 0, 114, 0, 111, 0, 117, 0, 112, 0, 32, 0, 110, 0, 97, 0, 109, 0,
                101, 0, 0, 0, 1, 0, 0, 0, 34, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0, 32,
                0, 103, 0, 114, 0, 101, 0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2, 0, 1, 0,
                0, 0, 38, 0, 9, 0, 100, 0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0, 100, 0, 0,
                82, 71, 66, 32, 63, 0, 0, 0, 62, 153, 153, 154, 61, 204, 204, 205, 0, 2, 192, 2
            ])
            .unwrap()
        );
    }

    #[test]
    fn it_reads_empty_name_correctly() {
        let group = Group::new(
            "".to_owned(),
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
        assert_eq!(
            group,
            Group::parse(&[
                0, 1, 0, 0, 0, 1, 0, 0, 0, 34, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0,
                32, 0, 103, 0, 114, 0, 101, 0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2, 0,
                1, 0, 0, 0, 38, 0, 9, 0, 100, 0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0, 100,
                0, 0, 82, 71, 66, 32, 63, 0, 0, 0, 62, 153, 153, 154, 61, 204, 204, 205, 0, 2, 192,
                2
            ])
            .unwrap()
        );
    }

    #[test]
    fn it_returns_error_on_empty_input() {
        let parser_result = Group::parse(&[]);
        assert!(
            matches!(parser_result.err(), Some(ASEError::InputDataParseError)),
            "Only ASEError::InputDataParseError should be returned"
        );
    }

    #[test]
    fn it_returns_error_on_length_larger_than_input() {
        // try to parse more name bytes than available
        let parser_result = Group::parse(&[12, 34]);
        assert!(
            matches!(parser_result.err(), Some(ASEError::InputDataParseError)),
            "Only ASEError::InputDataParseError should be returned"
        );
    }

    #[test]
    fn it_returns_error_on_invalid_utf_16() {
        // `[0xDC, 0x00]` is invalid utf16
        let parser_result = Group::parse(&[0, 5, 0xDC, 0x00, 0, 97, 0, 109, 0, 101]);
        assert!(
            matches!(parser_result.err(), Some(ASEError::UTF16Error)),
            "Only ASEError::UTF16Error should be returned"
        );
    }

    #[test]
    fn it_returns_error_on_invalid_block_type() {
        // 0, 255 is not a valid block type
        let parser_result = Group::parse(&[0, 1, 0, 0, 0, 255]);
        assert!(
            matches!(parser_result.err(), Some(ASEError::BlockTypeError)),
            "Only ASEError::BlockTypeError should be returned"
        );
    }

    #[test]
    fn it_returns_on_invalid_block_type() {
        let group = Group::new(
            "".to_owned(),
            vec![ColorBlock::new(
                "light grey".to_owned(),
                ColorValue::Gray(0.5),
                ColorType::Normal,
            )],
        );
        assert_eq!(
            group,
            Group::parse(&[
                0, 1, 0, 0, 0, 1, 0, 0, 0, 34, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0,
                32, 0, 103, 0, 114, 0, 101, 0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2,
                // second block, start with 0xc001 (GroupStart) instead of 0x0001 (ColorEntry), so
                // that parser stops early
                0xc0, 1, 0, 0, 0, 38, 0, 9, 0, 100, 0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0,
                100, 0, 0, 82, 71, 66, 32, 63, 0, 0, 0, 62, 153, 153, 154, 61, 204, 204, 205, 0, 2,
                192, 2
            ])
            .unwrap()
        );
    }

    #[test]
    fn it_returns_on_invalid_block() {
        let group = Group::new(
            "".to_owned(),
            vec![ColorBlock::new(
                "light grey".to_owned(),
                ColorValue::Gray(0.5),
                ColorType::Normal,
            )],
        );
        assert_eq!(
            group,
            Group::parse(&[
                0, 1, 0, 0, 0, 1, 0, 0, 0, 34, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0,
                32, 0, 103, 0, 114, 0, 101, 0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2,
                // second block, invalid, since half of it is missing
                0, 1, 0, 0, 0, 38, 0, 9, 0, 100, 0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0,
                100, 0, 0, 82, 71, 66, 32, 63, 0, 0
            ])
            .unwrap()
        );
    }

    #[test]
    fn it_returns_error_on_invalid_block_length() {
        let parser_result = Group::parse(&[
            //has block length of `34`, replacing it with invalid length of 13
            0, 1, 0, 0, 0, 1, 0, 0, 0, 13, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0, 32, 0,
            103, 0, 114, 0, 101, 0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0,
            38, 0, 9, 0, 100, 0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0, 100, 0, 0, 82, 71,
            66, 32, 63, 0, 0, 0, 62, 153, 153, 154, 61, 204, 204, 205, 0, 2, 192, 2,
        ]);
        assert!(parser_result.is_err());
        assert!(
            matches!(parser_result.err(), Some(ASEError::BlockTypeError)),
            "Only ASEError::InputDataParseError should be returned"
        );
    }

    #[test]
    fn it_returns_block_on_invalid_block_length() {
        let group = Group::new(
            "".to_owned(),
            vec![ColorBlock::new(
                "light grey".to_owned(),
                ColorValue::Gray(0.5),
                ColorType::Normal,
            )],
        );
        let parser_result = Group::parse(&[
            //has block length of `34`, replacing it with invalid length of 130
            0, 1, 0, 0, 0, 1, 0, 0, 0, 130, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0, 32, 0,
            103, 0, 114, 0, 101, 0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0,
            38, 0, 9, 0, 100, 0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0, 100, 0, 0, 82, 71,
            66, 32, 63, 0, 0, 0, 62, 153, 153, 154, 61, 204, 204, 205, 0, 2, 192, 2,
        ]);
        assert!(parser_result.is_ok());
        assert_eq!(group, parser_result.unwrap());
    }
}
