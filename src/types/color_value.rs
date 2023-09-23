use crate::buffer::Buffer;

/// Color data
#[derive(Debug, Clone)]
pub enum ColorValue {
    Cmyk(f32, f32, f32, f32),
    Rgb(f32, f32, f32),
    Lab(f32, f32, f32),
    Gray(f32),
}

impl ColorValue {
    /// Returns the color type identifier
    pub(super) fn get_type(&self) -> &[u8] {
        match self {
            ColorValue::Cmyk(_, _, _, _) => b"CMYK",
            ColorValue::Rgb(_, _, _) => b"RGB ",
            ColorValue::Lab(_, _, _) => b"LAB ",
            ColorValue::Gray(_) => b"Gray",
        }
    }

    /// Write the color values to the given [`Buffer`]
    pub(super) fn write_values(self, buf: &mut Buffer) {
        match self {
            ColorValue::Cmyk(c, m, y, k) => {
                buf.write_f32(c);
                buf.write_f32(m);
                buf.write_f32(y);
                buf.write_f32(k);
            }
            ColorValue::Rgb(r, g, b) => {
                buf.write_f32(r);
                buf.write_f32(g);
                buf.write_f32(b);
            }
            ColorValue::Lab(l, a, b) => {
                buf.write_f32(l);
                buf.write_f32(a);
                buf.write_f32(b);
            }
            ColorValue::Gray(value) => buf.write_f32(value),
        }
    }

    /// Calculate the length of the color
    ///
    /// The length is based on the number of f32, times 4,
    /// as each one requires 4 bytes.
    pub(super) fn calculate_length(&self) -> u32 {
        match self {
            ColorValue::Cmyk(_, _, _, _) => 16,
            ColorValue::Rgb(_, _, _) | ColorValue::Lab(_, _, _) => 12,
            ColorValue::Gray(_) => 4,
        }
    }
}
