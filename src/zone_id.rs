//! Having a bit of fun storing zone IDs in-memory as compactly as possible.

use anyhow::{Error, Result};
use std::fmt::Display;

/// CloudFlare Zone IDs are 16-byte unique IDs
pub struct ZoneId {
    id: [u8; 16],
}

impl ZoneId {
    pub fn new(hex_string: &str) -> Result<Self> {
        // Ensure the input length is exactly 32 characters (16 bytes)
        if hex_string.len() != 32 {
            return Err(Error::msg("Hex string must be 32 characters long"));
        }
        let mut bytes = [0u8; 16];
        #[allow(clippy::needless_range_loop)]
        for i in 0..16 {
            // Each byte in hexadecimal is encoded in two string characters.
            let slice_start = i * 2;
            let slice_end = slice_start + 2;
            let hex_byte = &hex_string[slice_start..slice_end];

            match u8::from_str_radix(hex_byte, 16) {
                Ok(byte) => bytes[i] = byte,
                Err(_) => {
                    return Err(Error::msg(format!(
                        "Invalid hex byte at position {i}: {hex_byte}"
                    )))
                }
            }
        }

        Ok(Self { id: bytes })
    }
}

impl Display for ZoneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.id {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_io() {
        let id_str = "30ed3e88cd9a56e0eb2b326a63500f4e";
        let zid = ZoneId::new(id_str).expect("zid can be constructed");
        let output = zid.to_string();
        assert_eq!(id_str, &output);
    }
    #[test]
    fn test_zone_id_wrong_length() {
        let zid = ZoneId::new("30ed3e88cd9a56e0eb2b326a63500fe");
        assert!(zid.is_err());
        if let Err(e) = zid {
            let msg = e.to_string();
            assert_eq!(msg, "Hex string must be 32 characters long");
        }
        let zid = ZoneId::new("30ed3e88cd9a56e0eb2b326a63500feee");
        assert!(zid.is_err());
        if let Err(e) = zid {
            let msg = e.to_string();
            assert_eq!(msg, "Hex string must be 32 characters long");
        }
    }
    #[test]
    fn test_invalid_hex() {
        let zid = ZoneId::new("00zzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
        assert!(zid.is_err());
        if let Err(e) = zid {
            let msg = e.to_string();
            assert_eq!(msg, "Invalid hex byte at position 1: zz");
        }
        let zid = ZoneId::new("0000zzzzzzzzzzzzzzzzzzzzzzzzzzzz");
        assert!(zid.is_err());
        if let Err(e) = zid {
            let msg = e.to_string();
            assert_eq!(msg, "Invalid hex byte at position 2: zz");
        }
    }
}
