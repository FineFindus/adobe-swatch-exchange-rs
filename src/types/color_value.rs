use crate::{buffer::Buffer, error::ASEError};

/// Color data
#[derive(Debug, Clone, PartialEq)]
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

impl TryFrom<&[u8]> for ColorValue {
    type Error = ASEError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match &value[..4] {
            b"CMYK" => {
                let cyan = f32::from_be_bytes(value[4..8].try_into()?);
                let magenta = f32::from_be_bytes(value[8..12].try_into()?);
                let yellow = f32::from_be_bytes(value[12..16].try_into()?);
                let black = f32::from_be_bytes(value[16..20].try_into()?);
                Ok(ColorValue::Cmyk(cyan, magenta, yellow, black))
            }
            b"RGB " => {
                let red = f32::from_be_bytes(value[4..8].try_into()?);
                let green = f32::from_be_bytes(value[8..12].try_into()?);
                let blue = f32::from_be_bytes(value[12..16].try_into()?);
                Ok(ColorValue::Rgb(red, green, blue))
            }
            b"LAB " => {
                let l = f32::from_be_bytes(value[4..8].try_into()?);
                let a = f32::from_be_bytes(value[8..12].try_into()?);
                let b = f32::from_be_bytes(value[12..16].try_into()?);
                Ok(ColorValue::Lab(l, a, b))
            }
            b"Gray" => Ok(ColorValue::Gray(f32::from_be_bytes(
                value[4..8].try_into()?,
            ))),
            _ => Err(ASEError::Invalid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_cmyk() {
        let rgb = ColorValue::Cmyk(0.0, 49.0, 54.0, 25.0);
        let mut buffer = Buffer::with_capacity(20);
        buffer.write_slice(rgb.get_type());
        rgb.clone().write_values(&mut buffer);
        let res = ColorValue::try_from(buffer.into_vec().as_slice());
        assert!(res.is_ok());
        assert_eq!(rgb, res.unwrap());
    }

    #[test]
    fn it_parses_rgb() {
        let rgb = ColorValue::Rgb(0.7490196078431373, 0.3803921568627451, 0.41568627450980394);
        let mut buffer = Buffer::with_capacity(20);
        buffer.write_slice(rgb.get_type());
        rgb.clone().write_values(&mut buffer);
        let res = ColorValue::try_from(buffer.into_vec().as_slice());
        assert!(res.is_ok());
        assert_eq!(rgb, res.unwrap());
    }

    #[test]
    fn it_parses_lab() {
        let rgb = ColorValue::Lab(52.5823974609375, 38.5067749023437, 12.4209403991699);
        let mut buffer = Buffer::with_capacity(20);
        buffer.write_slice(rgb.get_type());
        rgb.clone().write_values(&mut buffer);
        let res = ColorValue::try_from(buffer.into_vec().as_slice());
        assert!(res.is_ok());
        assert_eq!(rgb, res.unwrap());
    }

    #[test]
    fn it_parses_gray() {
        let gray = ColorValue::Gray(0.7490196078431373);
        let mut buffer = Buffer::with_capacity(8);
        buffer.write_slice(gray.get_type());
        gray.clone().write_values(&mut buffer);
        let res = ColorValue::try_from(buffer.into_vec().as_slice());
        assert!(res.is_ok());
        assert_eq!(gray, res.unwrap());
    }
}
