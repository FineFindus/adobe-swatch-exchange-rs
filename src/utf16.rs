/// Truncates the given UTF-16 data to be at most `len` long.
///
/// # Examples
///
/// ```ignore
/// # use adobe_swatch_exchange::utf16::truncate_utf16;
/// let mut data = "hello".encode_utf16().collect::<Vec<_>>();
/// truncate_utf16(&mut data, 2);
/// assert_eq!(data, "he".encode_utf16().collect::<Vec<_>>());
/// ```
pub(crate) fn truncate_utf16(data: &mut Vec<u16>, len: usize) {
    if data.len() <= len {
        return;
    }

    let mut end = len;
    if end > 0 && (0xD800..0xDC00).contains(&data[end - 1]) {
        end -= 1;
    }

    data.truncate(end);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_empty_input() {
        let mut data = Vec::new();

        truncate_utf16(&mut data, 10);

        assert!(data.is_empty());
    }

    #[test]
    fn truncate_handles_short_input() {
        let mut data = "hello".encode_utf16().collect::<Vec<_>>();
        let original = data.clone();

        truncate_utf16(&mut data, 10);

        assert_eq!(data, original);
    }

    #[test]
    fn truncates_input_to_maximum_length() {
        let mut data = vec![b'a' as u16; 10];

        truncate_utf16(&mut data, 5);

        assert_eq!(data.len(), 5);
    }

    #[test]
    fn removes_trailing_high_surrogate() {
        let mut data = vec![b'A' as u16, 0xD800, 0xDC00];

        truncate_utf16(&mut data, 2);

        assert_eq!(data, vec![b'A' as u16]);
    }
}
