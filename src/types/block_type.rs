/// Block indentifier
#[derive(Debug, Clone)]
pub(crate) enum BlockType {
    /// Indicates the start of a [`Group`]
    GroupStart = 0xc001,
    /// Indicates the end of a [`Group`]
    GroupEnd = 0xc002,
    /// Indicates the start of a [`ColorBlock`]
    ColorEntry = 0x0001,
}
