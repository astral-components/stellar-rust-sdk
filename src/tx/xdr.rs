//! Minimal Stellar XDR writer for the transaction types this SDK encodes.

/// Growable XDR buffer (big-endian, 4-byte aligned).
#[derive(Debug, Default, Clone)]
pub struct XdrWriter {
    buf: Vec<u8>,
}

impl XdrWriter {
    /// Empty writer.
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Finished XDR bytes.
    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    /// Borrow the encoded bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    /// Write a 4-byte aligned `i32`.
    pub fn write_i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    /// Write a 4-byte aligned `u32`.
    pub fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    /// Write an `i64`.
    pub fn write_i64(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    /// Write a `u64`.
    pub fn write_u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    /// Write an XDR boolean (`true` = 1).
    pub fn write_bool(&mut self, v: bool) {
        self.write_i32(i32::from(v));
    }

    /// Write a fixed-length opaque (already a multiple of 4, or padded).
    pub fn write_opaque_fixed(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
        let pad = (4 - (bytes.len() % 4)) % 4;
        self.buf.extend(std::iter::repeat(0).take(pad));
    }

    /// Write a variable-length opaque (`opaque<>`).
    pub fn write_opaque_var(&mut self, bytes: &[u8]) {
        self.write_u32(bytes.len() as u32);
        self.write_opaque_fixed(bytes);
    }

    /// Write an XDR string (`string<>`).
    pub fn write_string(&mut self, s: &str) {
        self.write_opaque_var(s.as_bytes());
    }

    /// `PublicKey` / `AccountID` (type ED25519 + 32 bytes).
    pub fn write_account_id(&mut self, ed25519: &[u8; 32]) {
        self.write_i32(0); // PUBLIC_KEY_TYPE_ED25519
        self.write_opaque_fixed(ed25519);
    }

    /// `MuxedAccount` with `KEY_TYPE_ED25519`.
    pub fn write_muxed_ed25519(&mut self, ed25519: &[u8; 32]) {
        self.write_i32(0); // KEY_TYPE_ED25519
        self.write_opaque_fixed(ed25519);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_opaque_to_four_bytes() {
        let mut w = XdrWriter::new();
        w.write_opaque_var(b"ab");
        // length (4) + 'a' 'b' 0 0
        assert_eq!(w.as_slice(), &[0, 0, 0, 2, b'a', b'b', 0, 0]);
    }

    #[test]
    fn integers_are_big_endian() {
        let mut w = XdrWriter::new();
        w.write_i32(0x01020304);
        assert_eq!(w.as_slice(), &[1, 2, 3, 4]);
    }
}
