use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};
use std::cmp;

use crate::{GenericNumber, GenericFormatter, GenericFormatterImpl, Integer, IntegerRenderer, IntegerRendererImpl};

/// Render a [`GenericNumber`] as a binary value.
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create a GenericNumber directly - normally you'd use a IntegerReader
/// let number = GenericNumber::from(15u8);
///
/// // Default 'pretty' formatter
/// assert_eq!("0b00001111", BinaryFormatter::pretty().render(number).unwrap());
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BinaryFormatter {
    /// Prefix binary strings with `0b`
    pub prefix: bool,

    /// Zero-pad binary strings to the full width - `00000001` vs `1`
    ///
    /// Unlike `min_digits`, this pads to the variable size, not an arbitrary
    /// size.
    pub padded: bool,

    /// Print at least this many bits (similar to padded, but user controlled)
    ///
    /// This is useful for, say, a bitmask that requires 12 bits to represent.
    /// Unlike `padded`, this pads to a custom size, not related to the
    /// variable size.
    pub min_digits: usize,
}

impl BinaryFormatter {
    pub fn new(prefix: bool, padded: bool) -> GenericFormatter {
        GenericFormatter::Binary(Self {
            prefix: prefix,
            padded: padded,
            min_digits: 0,
        })
    }

    pub fn new_with_min_size(prefix: bool, min_digits: usize) -> GenericFormatter {
        GenericFormatter::Binary(Self {
            prefix: prefix,
            padded: false,
            min_digits: min_digits,
        })
    }

    pub fn pretty() -> GenericFormatter {
        Self::new(true, true)
    }

    pub fn new_integer(prefix: bool, padded: bool) -> IntegerRenderer {
        IntegerRenderer::Binary(Self {
            prefix: prefix,
            padded: padded,
            min_digits: 0,
        })
    }

    pub fn new_with_min_size_integer(prefix: bool, min_digits: usize) -> IntegerRenderer {
        IntegerRenderer::Binary(Self {
            prefix: prefix,
            padded: false,
            min_digits: min_digits,
        })
    }

    pub fn pretty_integer() -> IntegerRenderer {
        Self::new_integer(true, true)
    }
}

impl GenericFormatterImpl for BinaryFormatter {
    fn render(&self, number: GenericNumber) -> SimpleResult<String> {
        let mut s = match (self.padded, number) {
            (true, GenericNumber::U8(v))   => format!("{:08b}", v),
            (true, GenericNumber::U16(v))  => format!("{:016b}", v),
            (true, GenericNumber::U32(v))  => format!("{:032b}", v),
            (true, GenericNumber::U64(v))  => format!("{:064b}", v),
            (true, GenericNumber::U128(v)) => format!("{:0128b}", v),
            (true, GenericNumber::I8(v))   => format!("{:08b}", v),
            (true, GenericNumber::I16(v))  => format!("{:016b}", v),
            (true, GenericNumber::I32(v))  => format!("{:032b}", v),
            (true, GenericNumber::I64(v))  => format!("{:064b}", v),
            (true, GenericNumber::I128(v)) => format!("{:0128b}", v),

            (false, GenericNumber::U8(v))   => format!("{:b}", v),
            (false, GenericNumber::U16(v))  => format!("{:b}", v),
            (false, GenericNumber::U32(v))  => format!("{:b}", v),
            (false, GenericNumber::U64(v))  => format!("{:b}", v),
            (false, GenericNumber::U128(v)) => format!("{:b}", v),
            (false, GenericNumber::I8(v))   => format!("{:b}", v),
            (false, GenericNumber::I16(v))  => format!("{:b}", v),
            (false, GenericNumber::I32(v))  => format!("{:b}", v),
            (false, GenericNumber::I64(v))  => format!("{:b}", v),
            (false, GenericNumber::I128(v)) => format!("{:b}", v),

            (_, GenericNumber::F32(_))      => bail!("Cannot display floating point as binary"),
            (_, GenericNumber::F64(_))      => bail!("Cannot display floating point as binary"),
            (_, GenericNumber::Char(_, _))  => bail!("Cannot display character as binary"),
        };

        // Pad if needed
        if self.min_digits > s.len() {
            s = format!("{}{}", str::repeat("0", self.min_digits - s.len()), s);
        }

        // Add the prefix after for simplicity
        if self.prefix {
            s = format!("0b{}", s);
        }

        Ok(s)
    }
}

impl IntegerRendererImpl for BinaryFormatter {
    fn render_integer(&self, number: Integer) -> String {
        match (self.padded, self.prefix) {
            (true,  false) => format!("{:0width$b}", number, width=cmp::max(self.min_digits, number.size() * 8)), // *8 because it's bytes, not characters
            (false, false) => format!("{:0width$b}", number, width=self.min_digits), // Still pad to min_digits

            (true,  true)  => format!("{:#0width$b}", number, width=(cmp::max(self.min_digits, number.size() * 8) + 2)), // +2 because the width includes the 0b which we don't want
            (false, true)  => format!("{:#0width$b}", number, width=self.min_digits + 2), // Still pad to min_digits (+2 for the prefix)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;
    use crate::{Context, IntegerReader};

    #[test]
    fn test_binary_i8() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\xab\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index    prefix      padded   expected
            (   0,      true,       true,    "0b00000000"),
            (   1,      true,       true,    "0b00000000"),
            (   2,      true,       true,    "0b00010010"),
            (   3,      true,       true,    "0b10101011"),
            (   4,      true,       true,    "0b11111111"),
            (   5,      true,       true,    "0b11111111"),

            (   0,      false,      false,   "0"),
            (   1,      false,      false,   "0"),
            (   2,      false,      false,   "10010"),
            (   3,      false,      false,   "10101011"),
            (   4,      false,      false,   "11111111"),
            (   5,      false,      false,   "11111111"),
        ];

        for (index, padded, prefix, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = IntegerReader::U8.read(context)?;

            assert_eq!(
                expected,
                BinaryFormatter::new_integer(prefix, padded).render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_minimum_length() -> SimpleResult<()> {
        // No prefix
        let number = IntegerReader::U8.read(Context::new(&b"\x01".to_vec()))?;
        assert_eq!("1",                                BinaryFormatter::new_with_min_size_integer(false, 0).render(number));
        assert_eq!("1",                                BinaryFormatter::new_with_min_size_integer(false, 1).render(number));
        assert_eq!("01",                               BinaryFormatter::new_with_min_size_integer(false, 2).render(number));
        assert_eq!("00000001",                         BinaryFormatter::new_with_min_size_integer(false, 8).render(number));
        assert_eq!("00000000000000000000000000000001", BinaryFormatter::new_with_min_size_integer(false, 32).render(number));

        // Prefix
        let number = IntegerReader::U8.read(Context::new(&b"\x01".to_vec()))?;
        assert_eq!("0b1",                                BinaryFormatter::new_with_min_size_integer(true, 0).render(number));
        assert_eq!("0b1",                                BinaryFormatter::new_with_min_size_integer(true, 1).render(number));
        assert_eq!("0b01",                               BinaryFormatter::new_with_min_size_integer(true, 2).render(number));
        assert_eq!("0b00000001",                         BinaryFormatter::new_with_min_size_integer(true, 8).render(number));
        assert_eq!("0b00000000000000000000000000000001", BinaryFormatter::new_with_min_size_integer(true, 32).render(number));

        // Zero
        let number = IntegerReader::U8.read(Context::new(&b"\x00".to_vec()))?;
        assert_eq!("0",                                BinaryFormatter::new_with_min_size_integer(false, 0).render(number));
        assert_eq!("0",                                BinaryFormatter::new_with_min_size_integer(false, 1).render(number));
        assert_eq!("00",                               BinaryFormatter::new_with_min_size_integer(false, 2).render(number));
        assert_eq!("00000000",                         BinaryFormatter::new_with_min_size_integer(false, 8).render(number));
        assert_eq!("00000000000000000000000000000000", BinaryFormatter::new_with_min_size_integer(false, 32).render(number));

        Ok(())
    }
}
