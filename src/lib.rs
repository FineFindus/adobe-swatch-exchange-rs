#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]

pub use error::{ASEError, ConformationError};
use types::{BlockType, GroupHold};
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
    let group_size: u32 = groups.iter().map(Group::calculate_length).sum();
    let color_size: u32 = colors.iter().map(ColorBlock::calculate_length).sum();
    // we slightly over-estimate the required amount of space here, to avoid a costly resizing
    let mut buf = buffer::Buffer::with_capacity((8 + group_size * 2 + color_size) as usize);

    // file metadata
    buf.write_slice(types::FILE_SIGNATURE);
    buf.write_u32(types::VERSION);
    // number of blocks
    buf.write_u32((groups.len() + colors.len()) as u32);

    // write groups
    groups.into_iter().for_each(|group| group.write(&mut buf));

    // write single colors
    colors.into_iter().for_each(|block| block.write(&mut buf));

    buf.into_vec()
}

/// Read groups and single colors from the `.ase` file.
///
/// # Errors
///
/// This function will return an error if either a read to the given data fails,
/// or the ASE file is invalid.
///
/// # Examples
/// ```rust
/// # use adobe_swatch_exchange::read_ase;
/// // any source
/// let source = vec![65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 0];
/// let (groups, colors) = read_ase(&*source).unwrap();
/// # assert_eq!((groups, colors), (vec![], vec![]));
/// ```
pub fn read_ase<T: std::io::Read>(mut ase: T) -> Result<(Vec<Group>, Vec<ColorBlock>), ASEError> {
    let mut buf_u32 = [0; 4];

    // read magic bytes
    ase.read_exact(&mut buf_u32)?;
    if &buf_u32 != types::FILE_SIGNATURE {
        return Err(ASEError::Invalid(error::ConformationError::FileSignature));
    }

    // read version, should be 1.0
    ase.read_exact(&mut buf_u32)?;
    if buf_u32 != types::VERSION.to_be_bytes() {
        return Err(ASEError::Invalid(error::ConformationError::FileVersion));
    }

    ase.read_exact(&mut buf_u32)?;
    let number_of_blocks = u32::from_be_bytes(buf_u32);

    let mut groups = Vec::new();
    let mut color_blocks = Vec::new();
    let mut buf_u16 = [0; 2];

    // temporary group to handle nonconforming group blocks
    let mut group_hold = GroupHold::Empty;
    let mut group_hold_value = Group::default();

    let mut blocks_to_read = number_of_blocks;

    // allow skipping of empty blocks when a group-end block has a size field
    let mut skipped = 0;
    let mut safe_to_skip = false;

    while blocks_to_read > 0 {
        ase.read_exact(&mut buf_u16)?;

        // only skip if the next two bytes were zero and we haven't skipped two already.
        if buf_u16 == [0, 0] && skipped < 2 && safe_to_skip {
            skipped += 1;
            continue;
        }

        let block_type = BlockType::try_from(u16::from_be_bytes(buf_u16))?;

        if block_type != BlockType::GroupEnd && group_hold == GroupHold::HoldingBuilt {
            return Err(ASEError::Invalid(error::ConformationError::GroupEnd));
        }

        // block length for GroupEnd blocks should always be zero, the `skipped`
        // variable above is intended to help us avoid the issue where the size
        // is specified.
        let block_length = if block_type == BlockType::GroupEnd {
            safe_to_skip = true;
            skipped = 0;
            0
        } else {
            ase.read_exact(&mut buf_u32)?;
            let block_length = u32::from_be_bytes(buf_u32);
            safe_to_skip = false;
            block_length
        };

        let mut block = vec![0; block_length as usize];
        ase.read_exact(&mut block)?;

        // parse block data and add it appropriate vec
        match block_type {
            BlockType::GroupStart => {
                let block = Group::parse(&block)?;
                if group_hold != GroupHold::Empty {
                    return Err(ASEError::Invalid(error::ConformationError::GroupEnd));
                }
                // if the parsed block has any sub-blocks then it has already been built
                // and only a group-end block may follow it. Otherwise we are free to
                // add colors as they appear until a group-end block is encountered.
                group_hold = if block.blocks.is_empty() {
                    GroupHold::HoldingBuilding
                } else {
                    blocks_to_read += 1;
                    GroupHold::HoldingBuilt
                };
                group_hold_value = block;
            }
            // read by the group end
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
                    GroupHold::HoldingBuilding => group_hold_value.blocks.push(block),
                    GroupHold::Empty => color_blocks.push(block),
                    GroupHold::HoldingBuilt => {
                        return Err(ASEError::Invalid(error::ConformationError::GroupEnd))
                    }
                }
            }
        };

        blocks_to_read -= 1;
    }

    // if we haven't saved the last group, even if no end was found, go ahead and add it.
    if group_hold == GroupHold::HoldingBuilding {
        groups.push(group_hold_value);
    }

    // if we received a built group, but it was terminated, it is an error.
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
    fn it_reads_group_and_single_color_with_explicit_group_end_size() {
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
        let input_ase_bytes = create_ase(vec![group.clone()], vec![block.clone()]);
        let mut modified_ase_bytes = vec![0; 0];

        // The following code modifies the generated ASE bytes so that
        // the GroupEnd block is followed by a four byte zero length
        // specifier. The slice lengths presented here are specific
        // to the test and not general numbers.
        // The layout, in this case, is:
        //      bytes 0   - 127       GroupStart, ColorBlocks, GroupEnd
        //      bytes 128 - end       Global Colors
        // The modified data has the layout:
        //      bytes 0   - 127       GroupStart, ColorBlocks, GroupEnd
        //      bytes 128 - 131       u32(0)
        //      bytes 132 - end       Global Colors
        modified_ase_bytes.extend_from_slice(&input_ase_bytes[..128]);
        modified_ase_bytes.extend_from_slice(&[0; 4]);
        modified_ase_bytes.extend_from_slice(&input_ase_bytes[128..]);
        let res = read_ase(&*input_ase_bytes);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, (vec![group], vec![block]));
    }

    #[test]
    fn it_reads_group_and_single_color_with_group_block_name_only_size() {
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
        let input_ase_bytes = create_ase(vec![group.clone()], vec![block.clone()]);
        let mut modified_ase_bytes = vec![0; 0];

        // The following code modifies the generated ASE bytes so that
        // the GroupStart block is sized to contain only the name
        // of the group. This requires that the total number of blocks
        // be updated to include the colors present in the group.
        // The slice lengths presented here are specific
        // to the test and not general numbers.
        // The layout, in this case, is:
        //      bytes 0   - 7         Header
        //      bytes 8   - 11        2 expected blocks
        //      bytes 12  - 13        GroupStart
        //      bytes 14  - 17        Group block size 108
        //      bytes 18  - end       Groups and Colors
        // The modified data has the layout:
        //      bytes 0   - 7         Header
        //      bytes 8   - 11        5 expected blocks
        //      bytes 12  - 13        GroupStart
        //      bytes 14  - 17        Group block size 24
        //      bytes 18  - end       Groups and Colors
        modified_ase_bytes.extend_from_slice(&input_ase_bytes[..8]);
        modified_ase_bytes.extend_from_slice(&(5_u32.to_be_bytes()));
        modified_ase_bytes.extend_from_slice(&input_ase_bytes[12..14]);
        modified_ase_bytes.extend_from_slice(&(24_u32.to_be_bytes()));
        modified_ase_bytes.extend_from_slice(&input_ase_bytes[18..]);
        let res = read_ase(&*modified_ase_bytes);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, (vec![group], vec![block]));
    }

    #[test]
    fn it_reads_group_and_single_color_with_group_block_name_only_size_and_explicit_group_end_size()
    {
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
        let input_ase_bytes = create_ase(vec![group.clone()], vec![block.clone()]);
        let mut modified_ase_bytes = vec![0; 0];

        // The following code modifies the generated ASE bytes so that
        // the GroupStart block is sized to contain only the name
        // of the group. This requires that the total number of blocks
        // be updated to include the colors present in the group.
        // The slice lengths presented here are specific
        // to the test and not general numbers.
        // The layout, in this case, is:
        //      bytes 0   - 7         Header
        //      bytes 8   - 11        2 expected blocks
        //      bytes 12  - 13        GroupStart
        //      bytes 14  - 17        Group block size 108
        //      bytes 18  - end       Groups and Colors
        // The modified data has the layout:
        //      bytes 0   - 7         Header
        //      bytes 8   - 11        5 expected blocks
        //      bytes 12  - 13        GroupStart
        //      bytes 14  - 17        Group block size 24
        //      bytes 18  - 127       Groups and Sub Colors
        //      bytes 128 - 131       u32(0)
        //      bytes 132 - end       Global colors
        modified_ase_bytes.extend_from_slice(&input_ase_bytes[..8]);
        modified_ase_bytes.extend_from_slice(&(5_u32.to_be_bytes()));
        modified_ase_bytes.extend_from_slice(&input_ase_bytes[12..14]);
        modified_ase_bytes.extend_from_slice(&(24_u32.to_be_bytes()));
        modified_ase_bytes.extend_from_slice(&input_ase_bytes[18..128]);
        modified_ase_bytes.extend_from_slice(&[0; 4]);
        modified_ase_bytes.extend_from_slice(&input_ase_bytes[128..]);
        let res = read_ase(&*modified_ase_bytes);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, (vec![group], vec![block]));
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
