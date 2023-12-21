#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]

pub use error::{ASEError, ConformationError};
use types::BlockType;
use types::GroupHold;
pub use types::{ColorBlock, ColorType, ColorValue, Group};

mod buffer;
mod error;
mod types;

/// Creates an Adobe Swatch Exchange (ASE) file.
///
/// # Examples
/// ```rust
/// # use adobe_swatch_exchange::ColorBlock;
/// # use adobe_swatch_exchange::ColorValue;
/// # use adobe_swatch_exchange::ColorType;
/// # use adobe_swatch_exchange::create_ase;
/// let color = ColorBlock::new("name".to_owned(), ColorValue::Gray(0.5), ColorType::Normal);
/// let ase = create_ase(vec![], vec![color]);
/// # assert_eq!( ase, vec![65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 22, 0, 5, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2]);
/// ```
pub fn create_ase(groups: Vec<Group>, colors: Vec<ColorBlock>) -> Vec<u8> {
    let mut buf = buffer::Buffer::with_capacity(12 + groups.capacity() + colors.capacity());

    //file metadata
    buf.write_slice(types::FILE_SIGNATURE);
    buf.write_u32(types::VERSION);
    //number of blocks
    buf.write_u32((groups.len() + colors.len()) as u32);

    //write groups
    groups.into_iter().for_each(|group| group.write(&mut buf));

    //write single colors
    colors.into_iter().for_each(|block| block.write(&mut buf));

    buf.into_vec()
}

/// Read groups and single colors from the .ase file.
///
/// # Errors
///
/// This function will return an error if either a read to the given data fails,
/// or the ASE file is invalid.
///
/// # Examples
/// ```rust
/// # use adobe_swatch_exchange::read_ase;
/// //any source
/// let source = vec![65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 0];
/// let (groups, colors) = read_ase(&*source).unwrap();
/// # assert_eq!((groups, colors), (vec![], vec![]));
/// ```
pub fn read_ase<T: std::io::Read>(mut ase: T) -> Result<(Vec<Group>, Vec<ColorBlock>), ASEError> {
    let mut buf_u32 = [0; 4];

    //read magic bytes
    ase.read_exact(&mut buf_u32)?;
    if &buf_u32 != types::FILE_SIGNATURE {
        return Err(ASEError::Invalid(error::ConformationError::FileSignature));
    }

    //read version,should be 1.0
    ase.read_exact(&mut buf_u32)?;
    if buf_u32 != types::VERSION.to_be_bytes() {
        return Err(ASEError::Invalid(error::ConformationError::FileVersion));
    }

    ase.read_exact(&mut buf_u32)?;
    let number_of_blocks = u32::from_be_bytes(buf_u32);

    let mut groups = Vec::new();
    let mut color_blocks = Vec::new();
    let mut buf_u16 = [0; 2];

    //temporary group to handle nonconformant group blocks
    let mut group_hold = GroupHold::Empty;
    let mut group_hold_value = Group::default();

    let mut blocks_to_read = number_of_blocks;

    while blocks_to_read > 0 {
        ase.read_exact(&mut buf_u16)?;

        if buf_u16 == [0, 0] {
            continue;
        }

        let block_type = BlockType::try_from(u16::from_be_bytes(buf_u16))?;

        if block_type != BlockType::GroupEnd && group_hold == GroupHold::HoldingBuilt {
            return Err(ASEError::Invalid(error::ConformationError::GroupEnd));
        }

        let block_length = if block_type != BlockType::GroupEnd {
            ase.read_exact(&mut buf_u32)?;
            let block_length = u32::from_be_bytes(buf_u32);
            block_length
        } else {
            0
        };

        let mut block = vec![0; block_length as usize];
        ase.read_exact(&mut block)?;

        //parse block data and add it appropriate vec
        match block_type {
            BlockType::GroupStart => {
                let block = Group::parse(&block)?;
                if group_hold != types::GroupHold::Empty {
                    return Err(ASEError::Invalid(error::ConformationError::GroupEnd));
                }
                group_hold = if block.blocks.len() == 0 {
                    types::GroupHold::HoldingBuilding
                } else {
                    blocks_to_read += 1;
                    types::GroupHold::HoldingBuilt
                };
                group_hold_value = block;
            }
            //read by the group end
            BlockType::GroupEnd => match group_hold {
                GroupHold::HoldingBuilding | GroupHold::HoldingBuilt => {
                    groups.push(group_hold_value.clone());
                    group_hold = GroupHold::Empty;
                }
                GroupHold::Empty => {
                    return Err(ASEError::Invalid(error::ConformationError::GroupEnd))
                }
            },
            BlockType::ColorEntry => {
                let block = ColorBlock::parse(&block)?;
                match group_hold {
                    types::GroupHold::HoldingBuilding => group_hold_value.blocks.push(block),
                    types::GroupHold::Empty => color_blocks.push(block),
                    types::GroupHold::HoldingBuilt => {
                        return Err(ASEError::Invalid(error::ConformationError::GroupEnd))
                    }
                }
            }
        };

        blocks_to_read -= 1;
    }

    if group_hold == GroupHold::HoldingBuilding {
        groups.push(group_hold_value);
    }

    if group_hold == GroupHold::HoldingBuilt {
        return Err(ASEError::Invalid(error::ConformationError::GroupEnd));
    }

    Ok((groups, color_blocks))
}

#[cfg(test)]
mod tests {
    use crate::error::ConformationError;

    use super::*;

    #[test]
    fn it_writes_empty_args() {
        assert_eq!(
            create_ase(vec![], vec![]),
            vec![65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 0]
        )
    }

    #[test]
    fn it_writes_single_color() {
        let block = ColorBlock::new("name".to_owned(), ColorValue::Gray(0.5), ColorType::Normal);
        assert_eq!(
            create_ase(vec![], vec![block]),
            vec![
                65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 22, 0, 5, 0, 110, 0, 97, 0,
                109, 0, 101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2
            ]
        )
    }

    #[test]
    fn it_writes_group_color() {
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
            create_ase(vec![group], vec![]),
            vec![
                65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 1, 192, 1, 0, 0, 0, 108, 0, 11, 0, 103, 0,
                114, 0, 111, 0, 117, 0, 112, 0, 32, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 0, 1, 0,
                0, 0, 34, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0, 32, 0, 103, 0, 114, 0,
                101, 0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 38, 0, 9, 0,
                100, 0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0, 100, 0, 0, 82, 71, 66, 32,
                63, 0, 0, 0, 62, 153, 153, 154, 61, 204, 204, 205, 0, 2, 192, 2
            ]
        )
    }

    #[test]
    fn it_writes_group_and_single_color() {
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
        let block = ColorBlock::new("name".to_owned(), ColorValue::Gray(0.5), ColorType::Normal);
        assert_eq!(
            create_ase(vec![group], vec![block]),
            vec![
                65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 2, 192, 1, 0, 0, 0, 108, 0, 11, 0, 103, 0,
                114, 0, 111, 0, 117, 0, 112, 0, 32, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 0, 1, 0,
                0, 0, 34, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0, 32, 0, 103, 0, 114, 0,
                101, 0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 38, 0, 9, 0,
                100, 0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0, 100, 0, 0, 82, 71, 66, 32,
                63, 0, 0, 0, 62, 153, 153, 154, 61, 204, 204, 205, 0, 2, 192, 2, 0, 1, 0, 0, 0, 22,
                0, 5, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2
            ]
        )
    }

    #[test]
    fn it_reads_empty() {
        let res = read_ase(&*vec![65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 0]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (vec![], vec![]));
    }

    #[test]
    fn it_reads_single_color() {
        let block = ColorBlock::new("name".to_owned(), ColorValue::Gray(0.5), ColorType::Normal);
        let res = read_ase(&*create_ase(vec![], vec![block.clone()]));
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, (vec![], vec![block]));
        assert_eq!(res.1.first().unwrap().name, "name".to_owned());
    }

    #[test]
    fn it_reads_group() {
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
        let res = read_ase(&*create_ase(vec![group.clone()], vec![]));
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, (vec![group], vec![]));
        assert_eq!(res.0.first().unwrap().name, "group name".to_owned());
    }

    #[test]
    fn it_reads_group_and_single_color() {
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
        let block = ColorBlock::new("name".to_owned(), ColorValue::Gray(0.5), ColorType::Normal);
        let res = read_ase(&*create_ase(vec![group.clone()], vec![block.clone()]));
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, (vec![group], vec![block]));
        assert_eq!(res.0.first().unwrap().name, "group name".to_owned());
        assert_eq!(res.1.first().unwrap().name, "name".to_owned());
    }

    #[test]
    fn it_returns_incorrect_block_type_error() {
        let input_bad_block_type = vec![
            65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 22, 0, 5, 0, 110, 0, 97, 0, 109,
            0, 101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2,
        ];
        let parser_result = read_ase(&*input_bad_block_type);
        assert!(
            parser_result.is_err(),
            "Parser result must be an error with an invalid block type."
        );
        assert!(
            matches!(parser_result.err(), Some(ASEError::BlockTypeError)),
            "Expected bad block type error"
        );
    }

    #[test]
    fn it_returns_incorrect_color_type_error() {
        let input_bad_color_type = vec![
            65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 22, 0, 5, 0, 110, 0, 97, 0, 109,
            0, 101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 3,
        ];
        let parser_result = read_ase(&*input_bad_color_type);
        assert!(
            parser_result.is_err(),
            "Parser result must be an error with an invalid color type."
        );
        assert!(
            matches!(parser_result.err(), Some(ASEError::ColorTypeError)),
            "Expected bad color type error"
        );
    }

    #[test]
    fn it_returns_incorrect_color_format_error() {
        let input_bad_color_format = vec![
            65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 22, 0, 5, 0, 110, 0, 97, 0, 109,
            0, 101, 0, 0, 72, 114, 97, 121, 63, 0, 0, 0, 0, 3,
        ];
        let parser_result: Result<(Vec<Group>, Vec<ColorBlock>), ASEError> =
            read_ase(&*input_bad_color_format);
        assert!(
            parser_result.is_err(),
            "Parser result must be an error with an invalid color format."
        );
        assert!(
            matches!(parser_result.err(), Some(ASEError::ColorFormat)),
            "Expected bad color format error"
        );
    }

    #[test]
    fn it_returns_incorrect_signature_error() {
        let input_bad_signature = vec![65, 80, 69, 70, 1, 1, 0, 0, 0, 0, 0, 0];
        let parser_result = read_ase(&*input_bad_signature);
        assert!(
            parser_result.is_err(),
            "Parser result must be an error with an invalid signature."
        );
        assert!(
            matches!(
                parser_result.err(),
                Some(ASEError::Invalid(ConformationError::FileSignature))
            ),
            "Only ASEError::Invalid(error::ConformationError::FileSignature) should be returned"
        );
    }

    #[test]
    fn it_returns_incorrect_version_error() {
        let input_bad_file_version = vec![65, 83, 69, 70, 1, 1, 0, 0, 0, 0, 0, 0];
        let parser_result = read_ase(&*input_bad_file_version);
        assert!(
            parser_result.is_err(),
            "Parser result must be an error with an invalid file version."
        );
        assert!(
            matches!(
                parser_result.err(),
                Some(ASEError::Invalid(ConformationError::FileVersion))
            ),
            "Only ASEError::Invalid(error::ConformationError::FileVersion) should be returned"
        );
    }

    #[test]
    fn it_returns_incorrect_block_end_error() {
        let input_bad_group_end = vec![
            65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 2, 192, 1, 0, 0, 0, 108, 0, 11, 0, 103, 0, 114, 0,
            111, 0, 117, 0, 112, 0, 32, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 0, 1, 0, 0, 0, 34, 0,
            11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0, 32, 0, 103, 0, 114, 0, 101, 0, 121, 0,
            0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 38, 0, 9, 0, 100, 0, 97, 0, 114,
            0, 107, 0, 32, 0, 114, 0, 101, 0, 100, 0, 0, 82, 71, 66, 32, 63, 0, 0, 0, 62, 153, 153,
            154, 61, 204, 204, 205, 0, 2, 0, 1, 0, 1, 0, 0, 0, 22, 0, 5, 0, 110, 0, 97, 0, 109, 0,
            101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2,
        ];
        let parser_result = read_ase(&*input_bad_group_end);
        assert!(
            parser_result.is_err(),
            "Parser result must be an error with an invalid group end."
        );
        assert!(
            matches!(
                parser_result.err(),
                Some(ASEError::Invalid(ConformationError::GroupEnd))
            ),
            "Only ASEError::Invalid(error::ConformationError::GroupEnd) should be returned"
        );
    }
}
