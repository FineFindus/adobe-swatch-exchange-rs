const FILE_SIGNATURE: &[u8; 4] = b"ASEF";
const VERSION: u32 = 0x00010000;

#[derive(Debug, Clone)]
enum BlockType {
    GroupStart,
    GroupEnd,
    ColorEntry,
}

#[derive(Debug, Clone)]
enum ColorType {
    Global,
    Spot,
    Normal,
}

#[derive(Debug, Clone)]
enum ColorModel {
    CMYK,
    RGB,
    LAB,
    Gray,
}

#[derive(Debug, Clone)]
enum ColorValue {
    CMYK(f32, f32, f32, f32),
    RGB(f32, f32, f32),
    LAB(f32, f32, f32),
    Gray(f32),
}

#[derive(Debug, Clone)]
struct Block<'a> {
    pub block_type: BlockType,
    pub name: &'a str,
    pub color_type: ColorType,
    pub color: ColorType,
}

pub fn create_ase() -> Vec<u8> {
    let buf = Vec::new();
    buf.append(FILE_SIGNATURE);
    vec![]
}
