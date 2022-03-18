use serde::{Serialize, Deserialize};
use std::cmp;

use crate::{Integer, IntegerRenderer, IntegerRendererTrait};

/// Render an [`Integer`] as a binary value.
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Default 'pretty' formatter
/// assert_eq!("0b00001111", BinaryFormatter::new_pretty().render_integer(15u8));
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

impl Default for BinaryFormatter {
    fn default() -> Self {
        Self::new_pretty()
    }
}

impl From<BinaryFormatter> for IntegerRenderer {
    fn from(f: BinaryFormatter) -> IntegerRenderer {
        IntegerRenderer::Binary(f)
    }
}

impl BinaryFormatter {
    pub fn new(prefix: bool, padded: bool) -> Self {
        Self {
            prefix: prefix,
            padded: padded,
            min_digits: 0,
        }
    }

    pub fn new_with_min_size(prefix: bool, min_digits: usize) -> Self {
        Self {
            prefix: prefix,
            padded: false,
            min_digits: min_digits,
        }
    }

    pub fn new_pretty() -> Self {
        Self::new(true, true)
    }
}

impl IntegerRendererTrait for BinaryFormatter {
    fn render_integer(&self, number: impl Into<Integer>) -> String {
        let number: Integer = number.into();

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
                BinaryFormatter::new(prefix, padded).render_integer(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_binary_u24() -> SimpleResult<()> {
        let number = Integer::U24(0x0f, 0x0f0f);
        assert_eq!("000011110000111100001111", BinaryFormatter::new(false, true).render_integer(number));

        Ok(())
    }

    #[test]
    fn test_minimum_length() -> SimpleResult<()> {
        // No prefix
        let number = IntegerReader::U8.read(Context::new(&b"\x01".to_vec()))?;
        assert_eq!("1",                                BinaryFormatter::new_with_min_size(false, 0).render_integer(number));
        assert_eq!("1",                                BinaryFormatter::new_with_min_size(false, 1).render_integer(number));
        assert_eq!("01",                               BinaryFormatter::new_with_min_size(false, 2).render_integer(number));
        assert_eq!("00000001",                         BinaryFormatter::new_with_min_size(false, 8).render_integer(number));
        assert_eq!("00000000000000000000000000000001", BinaryFormatter::new_with_min_size(false, 32).render_integer(number));

        // Prefix
        let number = IntegerReader::U8.read(Context::new(&b"\x01".to_vec()))?;
        assert_eq!("0b1",                                BinaryFormatter::new_with_min_size(true, 0).render_integer(number));
        assert_eq!("0b1",                                BinaryFormatter::new_with_min_size(true, 1).render_integer(number));
        assert_eq!("0b01",                               BinaryFormatter::new_with_min_size(true, 2).render_integer(number));
        assert_eq!("0b00000001",                         BinaryFormatter::new_with_min_size(true, 8).render_integer(number));
        assert_eq!("0b00000000000000000000000000000001", BinaryFormatter::new_with_min_size(true, 32).render_integer(number));

        // Zero
        let number = IntegerReader::U8.read(Context::new(&b"\x00".to_vec()))?;
        assert_eq!("0",                                BinaryFormatter::new_with_min_size(false, 0).render_integer(number));
        assert_eq!("0",                                BinaryFormatter::new_with_min_size(false, 1).render_integer(number));
        assert_eq!("00",                               BinaryFormatter::new_with_min_size(false, 2).render_integer(number));
        assert_eq!("00000000",                         BinaryFormatter::new_with_min_size(false, 8).render_integer(number));
        assert_eq!("00000000000000000000000000000000", BinaryFormatter::new_with_min_size(false, 32).render_integer(number));

        Ok(())
    }
}
