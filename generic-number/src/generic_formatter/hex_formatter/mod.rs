use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::{GenericNumber, GenericFormatter, GenericFormatterImpl};

/// Render a [`GenericNumber`] as a hexadecimal value.
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create a GenericNumber directly - normally you'd use a GenericReader
/// let number = GenericNumber::from(0xaa2233u32);
///
/// // Default 'pretty' formatter
/// assert_eq!("0x00aa2233", HexFormatter::pretty().render(number).unwrap());
///
/// // Specify options: uppercase, no prefix, zero-padded
/// assert_eq!("00AA2233", HexFormatter::new(true,  false, true ).render(number).unwrap());
///
/// // Specify different options: lowercase, '0x' prefix, not padded
/// assert_eq!("0xaa2233", HexFormatter::new(false, true,  false).render(number).unwrap());
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HexFormatter {
    /// Print hex characters uppercase - `1A2B` vs `1a2b`.
    pub uppercase: bool,

    /// Prefix hex strings with `0x`
    pub prefix: bool,

    /// Zero-pad hex strings to the full width - `0001` vs `1`)
    pub padded: bool,
}


impl HexFormatter {
    pub fn new(uppercase: bool, prefix: bool, padded: bool) -> GenericFormatter {
        GenericFormatter::Hex(Self {
            uppercase: uppercase,
            prefix: prefix,
            padded: padded,
        })
    }

    pub fn pretty() -> GenericFormatter {
        Self::new(false, true, true)
    }
}

impl GenericFormatterImpl for HexFormatter {
    fn render(&self, number: GenericNumber) -> SimpleResult<String> {
        let mut s = match (self.padded, number) {
            (true, GenericNumber::U8(v))   => format!("{:02x}", v),
            (true, GenericNumber::U16(v))  => format!("{:04x}", v),
            (true, GenericNumber::U32(v))  => format!("{:08x}", v),
            (true, GenericNumber::U64(v))  => format!("{:016x}", v),
            (true, GenericNumber::U128(v)) => format!("{:032x}", v),
            (true, GenericNumber::I8(v))   => format!("{:02x}", v),
            (true, GenericNumber::I16(v))  => format!("{:04x}", v),
            (true, GenericNumber::I32(v))  => format!("{:08x}", v),
            (true, GenericNumber::I64(v))  => format!("{:016x}", v),
            (true, GenericNumber::I128(v)) => format!("{:032x}", v),

            (false, GenericNumber::U8(v))   => format!("{:x}", v),
            (false, GenericNumber::U16(v))  => format!("{:x}", v),
            (false, GenericNumber::U32(v))  => format!("{:x}", v),
            (false, GenericNumber::U64(v))  => format!("{:x}", v),
            (false, GenericNumber::U128(v)) => format!("{:x}", v),
            (false, GenericNumber::I8(v))   => format!("{:x}", v),
            (false, GenericNumber::I16(v))  => format!("{:x}", v),
            (false, GenericNumber::I32(v))  => format!("{:x}", v),
            (false, GenericNumber::I64(v))  => format!("{:x}", v),
            (false, GenericNumber::I128(v)) => format!("{:x}", v),

            (_, GenericNumber::F32(_))      => bail!("Cannot display floating point as hex"),
            (_, GenericNumber::F64(_))      => bail!("Cannot display floating point as hex"),
            (_, GenericNumber::Char(_, _))  => bail!("Cannot display character as hex"),
        };

        // Do uppercase after for simplicity
        if self.uppercase {
            s = s.to_uppercase();
        }

        // Likewise, do the prefix after
        if self.prefix {
            s = format!("0x{}", s);
        }

        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;
    use crate::{Context, Endian, GenericReader};

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
            let number = GenericReader::U8.read(context)?;

            assert_eq!(
                expected,
                HexFormatter::new(uppercase, prefix, padded).render(number)?,
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
            let number = GenericReader::U16(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                HexFormatter::new(uppercase, prefix, padded).render(number)?,
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
            let number = GenericReader::U32(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                HexFormatter::new(uppercase, prefix, padded).render(number)?,
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
            let number = GenericReader::U64(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                HexFormatter::new(uppercase, prefix, padded).render(number)?,
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
            let number = GenericReader::U64(Endian::Little).read(context)?;

            assert_eq!(
                expected,
                HexFormatter::new(uppercase, prefix, padded).render(number)?,
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
            let number = GenericReader::U128(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                HexFormatter::new(uppercase, prefix, padded).render(number)?,
            );
        }

        Ok(())
    }

}
