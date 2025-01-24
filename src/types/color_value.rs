use std::ops::Range;

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
                // ASE stores L* scaled to [0, 1]
                buf.write_f32(l / 100.0);
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
        let f32_from_bytes = |index: Range<usize>| {
            value
                .get(index)
                .ok_or(ASEError::InputDataParseError)
                .and_then(|data| {
                    <&[u8] as TryInto<[u8; 4]>>::try_into(data)
                        .map_err(|_| ASEError::InputDataParseError)
                })
                .map(f32::from_be_bytes)
        };

        match &value.get(..4) {
            Some(b"CMYK") => {
                let cyan = f32_from_bytes(4..8)?;
                let magenta = f32_from_bytes(8..12)?;
                let yellow = f32_from_bytes(12..16)?;
                let black = f32_from_bytes(16..20)?;
                Ok(ColorValue::Cmyk(cyan, magenta, yellow, black))
            }
            Some(b"RGB ") => {
                let red = f32_from_bytes(4..8)?;
                let green = f32_from_bytes(8..12)?;
                let blue = f32_from_bytes(12..16)?;
                Ok(ColorValue::Rgb(red, green, blue))
            }
            Some(b"LAB ") => {
                // scale L* to be in [0, 100]
                let l = f32_from_bytes(4..8)? * 100.0;
                let a = f32_from_bytes(8..12)?;
                let b = f32_from_bytes(12..16)?;
                Ok(ColorValue::Lab(l, a, b))
            }
            Some(b"Gray") => Ok(ColorValue::Gray(f32_from_bytes(4..8)?)),
            Some(_) => Err(ASEError::ColorFormat),
            _ => Err(ASEError::InputDataParseError),
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
        let rgb = ColorValue::Rgb(0.749_019_6, 0.380_392_16, 0.415_686_28);
        let mut buffer = Buffer::with_capacity(20);
        buffer.write_slice(rgb.get_type());
        rgb.clone().write_values(&mut buffer);
        let res = ColorValue::try_from(buffer.into_vec().as_slice());
        assert!(res.is_ok());
        assert_eq!(rgb, res.unwrap());
    }

    #[test]
    fn it_parses_lab() {
        let color = ColorValue::Lab(0.525_823_97, 38.506_775, 12.420_94);
        let mut buffer = Buffer::with_capacity(20);
        buffer.write_slice(color.get_type());
        color.clone().write_values(&mut buffer);
        let res = ColorValue::try_from(buffer.into_vec().as_slice());
        assert!(res.is_ok());
        assert_eq!(color, res.unwrap());
    }

    #[test]
    fn it_parses_gray() {
        let gray = ColorValue::Gray(0.749_019_6);
        let mut buffer = Buffer::with_capacity(8);
        buffer.write_slice(gray.get_type());
        gray.clone().write_values(&mut buffer);
        let res = ColorValue::try_from(buffer.into_vec().as_slice());
        assert!(res.is_ok());
        assert_eq!(gray, res.unwrap());
    }

    #[test]
    fn it_returns_input_data_parse_error() {
        let data = vec![];
        let res = ColorValue::try_from(&*data);
        assert!(
            matches!(res.err(), Some(ASEError::InputDataParseError)),
            "Only ASEError::InputDataParseError should be returned"
        );
    }

    #[test]
    fn it_returns_color_format_error() {
        let data = b"ABCD";
        let res = ColorValue::try_from(data.as_slice());
        assert!(
            matches!(res.err(), Some(ASEError::ColorFormat)),
            "Only ASEError::ColorFormat should be returned"
        );
    }

    #[test]
    fn it_returns_input_data_parse_error_for_oob() {
        // try check pass in data starting with the correct header, but ending too early
        let data = b"RGB sahdj";
        let res = ColorValue::try_from(data.as_slice());
        assert!(
            matches!(res.err(), Some(ASEError::InputDataParseError)),
            "Only ASEError::InputDataParseError should be returned"
        );
    }
}
