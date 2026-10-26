/// Represents an infinite buffer designed to hold individual bytes ([`u8`]).
///
/// It provides methods for easily writing other data types
/// as bytes. All bytes are written in big-endian byte order.
pub(crate) struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    /// Create a new Buffer with the specified capacity.
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Write the slice to [self].
    pub fn write_slice(&mut self, src: &[u8]) {
        self.data.extend_from_slice(src);
    }

    /// Write [u32] to [self].
    pub fn write_u32(&mut self, n: u32) {
        self.write_slice(&n.to_be_bytes());
    }

    /// Write [f32] to [self].
    pub fn write_f32(&mut self, n: f32) {
        self.write_slice(&n.to_be_bytes());
    }

    /// Write [u16] to [self].
    pub fn write_u16(&mut self, n: u16) {
        self.write_slice(&n.to_be_bytes());
    }

    /// Write a slice of [u16] to the buffer.
    pub fn write_u16_slice(&mut self, data: &[u16]) {
        for byte in data {
            self.write_u16(*byte);
        }
    }

    /// Returns the written buffer as a [`Vec<u8>`] of bytes.
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}
