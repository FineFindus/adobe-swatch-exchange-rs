pub use types::{ColorBlock, ColorType, ColorValue, Group};

mod buffer;
mod types;

/// Creates an Adobe Swatch Exchange (ASE) file.
///
///
///
/// # Examples
/// ```rust
///
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

    buf.to_vec()
}
