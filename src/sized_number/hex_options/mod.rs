use std::fmt::LowerHex;
use std::mem;

use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::sized_number::SizedOptions;

/// Configure display options for [`SizedDisplay::Hex`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HexOptions {
    /// Print hex characters uppercase - `1A2B` vs `1a2b`.
    pub uppercase: bool,

    /// Prefix hex strings with `0x`
    pub prefix: bool,

    /// Zero-pad hex strings to the full width - `0001` vs `1`)
    pub padded: bool,
}

impl Default for HexOptions {
    fn default() -> Self {
        Self {
            uppercase: false,
            prefix: true,
            padded: true,
        }
    }
}

impl HexOptions {
    /// An internal function to help with displaying hex.
    ///
    /// Unfortunately, I don't know of a way to require both [`UpperHex`] and
    /// [`LowerHex`] traits, so I do some manual formatting :-/
    // TODO make this non-pub
    pub fn to_s(self, v: Box<dyn LowerHex>) -> String {
        let v = v.as_ref();

        let mut h = match self.padded {
            // No padding is easy
            false => format!("{:x}",   v),

            // Padding requires a bit more tinkering to do dynamically
            true => {
                match (self.padded, mem::size_of_val(v) * 2) {
                    (true, 2)   => format!(  "{:02x}",  v),
                    (true, 4)   => format!(  "{:04x}",  v),
                    (true, 8)   => format!(  "{:08x}",  v),
                    (true, 16)  => format!(  "{:016x}", v),
                    (true, 32)  => format!(  "{:032x}", v),

                    // When not padded, or in doubt about length, just print normally
                    (_, _)      => format!(  "{:x}",     v),
                }
            }
        };

        // There's no way to make the parameter both LowerHex and UpperHex
        if self.uppercase {
            h = h.to_uppercase();
        }

        if self.prefix {
            h = format!("0x{}", h);
        }

        h
    }
}

impl SizedOptions for HexOptions {
    fn to_s_i8(&self, v:   i8)   -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_i16(&self, v:  i16)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_i32(&self, v:  i32)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_i64(&self, v:  i64)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_i128(&self, v: i128) -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_u8(&self, v:   u8)   -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_u16(&self, v:  u16)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_u32(&self, v:  u32)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_u64(&self, v:  u64)  -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_u128(&self, v: u128) -> SimpleResult<String> {
        Ok(self.to_s(Box::new(v)))
    }

    fn to_s_f32(&self, _v:  f32) -> SimpleResult<String> {
        bail!("Floating point values cannot be displayed as hex")
    }

    fn to_s_f64(&self, _v:  f64) -> SimpleResult<String> {
        bail!("Floating point values cannot be displayed as hex")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;
    use crate::sized_number::{Context, Endian, SizedDisplay, SizedDefinition};

    #[test]
    fn test_hex_u8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "0"),
            (   0,    true,       false,   false,    "0"),
            (   0,    false,      true,    false,    "0x0"),
            (   0,    false,      false,   true,     "00"),
            (   0,    true,       true,    true,     "0x00"),

            // index  uppercase   prefix   padded    expected
            (   1,    false,      false,   false,    "7f"),
            (   1,    true,       false,   false,    "7F"),
            (   1,    false,      true,    false,    "0x7f"),
            (   1,    false,      false,   true,     "7f"),
            (   1,    true,       true,    true,     "0x7F"),

            // index  uppercase   prefix   padded    expected
            (   2,    false,      false,   false,    "80"),
            (   2,    true,       false,   false,    "80"),
            (   2,    false,      true,    false,    "0x80"),
            (   2,    false,      false,   true,     "80"),
            (   2,    true,       true,    true,     "0x80"),

            // index  uppercase   prefix   padded    expected
            (   3,    false,      false,   false,    "ff"),
            (   3,    true,       false,   false,    "FF"),
            (   3,    false,      true,    false,    "0xff"),
            (   3,    false,      false,   true,     "ff"),
            (   3,    true,       true,    true,     "0xFF"),

        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U8.to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u16() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "0"),
            (   0,    true,       false,   false,    "0"),
            (   0,    false,      true,    false,    "0x0"),
            (   0,    false,      false,   true,     "0000"),
            (   0,    true,       true,    true,     "0x0000"),

            // index  uppercase   prefix   padded    expected
            (   2,    false,      false,   false,    "1234"),
            (   2,    true,       false,   false,    "1234"),
            (   2,    false,      true,    false,    "0x1234"),
            (   2,    false,      false,   true,     "1234"),
            (   2,    true,       true,    true,     "0x1234"),

            // index  uppercase   prefix   padded    expected
            (   4,    false,      false,   false,    "ffff"),
            (   4,    true,       false,   false,    "FFFF"),
            (   4,    false,      true,    false,    "0xffff"),
            (   4,    false,      false,   true,     "ffff"),
            (   4,    true,       true,    true,     "0xFFFF"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U16(Endian::Big).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u32() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "1234"),
            (   0,    true,       false,   false,    "1234"),
            (   0,    false,      true,    false,    "0x1234"),
            (   0,    false,      false,   true,     "00001234"),
            (   0,    true,       true,    true,     "0x00001234"),

            // index  uppercase   prefix   padded    expected
            (   4,    false,      false,   false,    "ffffffff"),
            (   4,    true,       false,   false,    "FFFFFFFF"),
            (   4,    false,      true,    false,    "0xffffffff"),
            (   4,    false,      false,   true,     "ffffffff"),
            (   4,    true,       true,    true,     "0xFFFFFFFF"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u64_big_endian() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "1234ffffffff"),
            (   0,    true,       false,   false,    "1234FFFFFFFF"),
            (   0,    false,      true,    false,    "0x1234ffffffff"),
            (   0,    false,      false,   true,     "00001234ffffffff"),
            (   0,    true,       true,    true,     "0x00001234FFFFFFFF"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u64_little_endian() -> SimpleResult<()> {
        let data = b"\x00\x12\x34\xFF\xFF\xFF\xFF\x00".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "ffffffff341200"),
            (   0,    true,       false,   false,    "FFFFFFFF341200"),
            (   0,    false,      true,    false,    "0xffffffff341200"),
            (   0,    false,      false,   true,     "00ffffffff341200"),
            (   0,    true,       true,    true,     "0x00FFFFFFFF341200"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U64(Endian::Little).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u128_big_endian() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00\x11\x22\x33\x44\x55\x66\x77\x88\x99\xaa\xbb\xcc\xdd\xee\xff".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "1"),
            (   0,    true,       false,   false,    "1"),
            (   0,    false,      true,    false,    "0x1"),
            (   0,    false,      false,   true,     "00000000000000000000000000000001"),
            (   0,    true,       true,    true,     "0x00000000000000000000000000000001"),

            // index  uppercase   prefix   padded    expected
            (   16,    false,      false,   false,    "112233445566778899aabbccddeeff"),
            (   16,    true,       false,   false,    "112233445566778899AABBCCDDEEFF"),
            (   16,    false,      true,    false,    "0x112233445566778899aabbccddeeff"),
            (   16,    false,      false,   true,     "00112233445566778899aabbccddeeff"),
            (   16,    true,       true,    true,     "0x00112233445566778899AABBCCDDEEFF"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U128(Endian::Big).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

}
