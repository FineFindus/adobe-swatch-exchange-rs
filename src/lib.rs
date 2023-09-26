pub use types::{ColorBlock, ColorType, ColorValue, Group};

mod buffer;
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
/// # assert_eq!( ase, vec![ 65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 18, 0, 5, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2 ]);
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
                65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 18, 0, 5, 0, 110, 0, 97, 0,
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
        let block = ColorBlock::new("name".to_owned(), ColorValue::Gray(0.5), ColorType::Normal);
        assert_eq!(
            create_ase(vec![group], vec![block]),
            vec![
                65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 2, 192, 1, 0, 0, 0, 72, 0, 11, 0, 103, 0, 114,
                0, 111, 0, 117, 0, 112, 0, 32, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 0, 1, 0, 0, 0,
                24, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0, 32, 0, 103, 0, 114, 0, 101,
                0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 30, 0, 9, 0, 100,
                0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0, 100, 0, 0, 82, 71, 66, 32, 63, 0,
                0, 0, 62, 153, 153, 154, 61, 204, 204, 205, 0, 2, 192, 2, 0, 1, 0, 0, 0, 18, 0, 5,
                0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2
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
                65, 83, 69, 70, 0, 1, 0, 0, 0, 0, 0, 2, 192, 1, 0, 0, 0, 72, 0, 11, 0, 103, 0, 114,
                0, 111, 0, 117, 0, 112, 0, 32, 0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 0, 1, 0, 0, 0,
                24, 0, 11, 0, 108, 0, 105, 0, 103, 0, 104, 0, 116, 0, 32, 0, 103, 0, 114, 0, 101,
                0, 121, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 30, 0, 9, 0, 100,
                0, 97, 0, 114, 0, 107, 0, 32, 0, 114, 0, 101, 0, 100, 0, 0, 82, 71, 66, 32, 63, 0,
                0, 0, 62, 153, 153, 154, 61, 204, 204, 205, 0, 2, 192, 2, 0, 1, 0, 0, 0, 18, 0, 5,
                0, 110, 0, 97, 0, 109, 0, 101, 0, 0, 71, 114, 97, 121, 63, 0, 0, 0, 0, 2
            ]
        )
    }
}
