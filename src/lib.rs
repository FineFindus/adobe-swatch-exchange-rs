#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]
pub use error::ASEError;
use types::BlockType;
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
        return Err(ASEError::Invalid);
    }

    //read version,should be 1.0
    ase.read_exact(&mut buf_u32)?;
    if buf_u32 != types::VERSION.to_be_bytes() {
        return Err(ASEError::Invalid);
    }

    ase.read_exact(&mut buf_u32)?;
    let number_of_blocks = u32::from_be_bytes(buf_u32);

    let mut groups = Vec::new();
    let mut color_blocks = Vec::new();
    let mut buf_u16 = [0; 2];

    for _ in 0..number_of_blocks {
        ase.read_exact(&mut buf_u16)?;
        let block_type = BlockType::try_from(u16::from_be_bytes(buf_u16))?;

        ase.read_exact(&mut buf_u32)?;
        let block_length = u32::from_be_bytes(buf_u32);

        let mut block = vec![0; block_length as usize];
        ase.read_exact(&mut block)?;

        //parse block data and add it appropriate vec
        match block_type {
            BlockType::GroupStart => {
                let block = Group::parse(&block)?;
                groups.push(block);

                // read the group end block
                ase.read_exact(&mut buf_u16)?;
                if BlockType::try_from(u16::from_be_bytes(buf_u16))? != BlockType::GroupEnd {
                    // group has no end, file is invalid
                    return Err(ASEError::Invalid);
                }
            }
            //read by the group end
            BlockType::GroupEnd => unreachable!(),
            BlockType::ColorEntry => {
                let block = ColorBlock::parse(&block)?;
                color_blocks.push(block);
            }
        };
    }

    Ok((groups, color_blocks))
}

#[cfg(test)]
mod tests {
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
}
