mod block_type;
mod color_block;
mod color_type;
mod color_value;
mod group;

pub use color_block::ColorBlock;
pub use color_type::ColorType;
pub use color_value::ColorValue;
pub use group::Group;

/// Magic Bytes for .ase files.
/// Equal to the bytes of `ASEF`.
pub(crate) const FILE_SIGNATURE: &[u8; 4] = b"ASEF";

/// Version of the ASE file.
pub(crate) const VERSION: u32 = 0x00010000;
