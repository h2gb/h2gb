use std::mem;
use serde::{Serialize, Deserialize};

use crate::{Integer, IntegerRenderer, IntegerRendererTrait};

/// Render an [`Integer`] as an octal value.
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create an Integer directly - normally you'd use a [`IntegerReader`]
/// let number = Integer::from(32u8);
///
/// // Default 'pretty' formatter
/// assert_eq!("0o40", OctalFormatter::pretty_integer().render(number));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OctalFormatter {
    /// Prefix octal strings with `0o`
    pub prefix: bool,

    /// Zero-pad octal strings to the full width - `0001` vs `1`)
    pub padded: bool,
}

impl OctalFormatter {
    pub fn new_integer(prefix: bool, padded: bool) -> IntegerRenderer {
        IntegerRenderer::Octal(Self {
            prefix: prefix,
            padded: padded,
        })
    }

    pub fn pretty_integer() -> IntegerRenderer {
        Self::new_integer(true, false)
    }
}

impl IntegerRendererTrait for OctalFormatter {
    fn render_integer(&self, number: Integer) -> String {
        if self.padded {
            // There might be a mathy way to get this, but /shrug
            let width = match number {
                Integer::U8(_)    => 3,
                Integer::U16(_)   => 6,
                Integer::U24(_,_) => 8,
                Integer::U32(_)   => 11,
                Integer::U64(_)   => 22,
                Integer::U128(_)  => 43,
                Integer::USize(_) => {
                    match mem::size_of::<usize>() {
                        1  => 3,
                        2  => 6,
                        4  => 11,
                        8  => 22,
                        16 => 43,
                        _ => 0,
                    }
                },

                Integer::I8(_)    => 3,
                Integer::I16(_)   => 6,
                Integer::I32(_)   => 11,
                Integer::I64(_)   => 22,
                // Integer::I128(_)  => 43,
                Integer::ISize(_) => {
                    match mem::size_of::<isize>() {
                        1  => 3,
                        2  => 6,
                        4  => 11,
                        8  => 22,
                        16 => 43,
                        _ => 0,
                    }
                },
            };

            match self.prefix {
                false => format!("{:0width$o}", number, width=width),
                true  => format!("{:#0width$o}", number, width=(width+2)),
            }
        } else {
            match self.prefix {
                false => format!("{:o}", number),
                true  => format!("{:#o}", number),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::{Context, Endian, IntegerReader};

    #[test]
    fn test_octal_u8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  prefix   padded   expected
            (   0,    false,   false,   "0"),
            (   1,    false,   false,   "177"),
            (   2,    false,   false,   "200"),
            (   3,    false,   false,   "377"),

            // index  prefix   padded   expected
            (   0,    false,   true,    "000"),
            (   1,    false,   true,    "177"),
            (   2,    false,   true,    "200"),
            (   3,    false,   true,    "377"),

            // index  prefix   padded   expected
            (   0,    true,    false,   "0o0"),
            (   1,    true,    false,   "0o177"),
            (   2,    true,    false,   "0o200"),
            (   3,    true,    false,   "0o377"),

            // index  prefix   padded   expected
            (   0,    true,    true,    "0o000"),
            (   1,    true,    true,    "0o177"),
            (   2,    true,    true,    "0o200"),
            (   3,    true,    true,    "0o377"),
        ];

        for (index, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = IntegerReader::U8.read(context)?;

            assert_eq!(
                expected,
                OctalFormatter::new_integer(prefix, padded).render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_octal_u16() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  prefix   padded   expected
            (   0,    false,   false,   "0"),
            (   2,    false,   false,   "11064"),
            (   4,    false,   false,   "177777"),

            // index  prefix   padded   expected
            (   0,    false,   true,    "000000"),
            (   2,    false,   true,    "011064"),
            (   4,    false,   true,    "177777"),

            // index  prefix   padded   expected
            (   0,    true,    false,   "0o0"),
            (   2,    true,    false,   "0o11064"),
            (   4,    true,    false,   "0o177777"),

            // index  prefix   padded   expected
            (   0,    true,    true,    "0o000000"),
            (   2,    true,    true,    "0o011064"),
            (   4,    true,    true,    "0o177777"),
        ];

        for (index, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = IntegerReader::U16(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                OctalFormatter::new_integer(prefix, padded).render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_octal_u32() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  prefix  padded   expected
            (   0,    false,  false,   "11064"),
            (   2,    false,  false,   "2215177777"),
            (   4,    false,  false,   "37777777777"),

            // index  prefix  padded   expected
            (   0,    false,  true,   "00000011064"),
            (   2,    false,  true,   "02215177777"),
            (   4,    false,  true,   "37777777777"),

            // index  prefix  padded   expected
            (   0,    true,  false,   "0o11064"),
            (   2,    true,  false,   "0o2215177777"),
            (   4,    true,  false,   "0o37777777777"),

            // index  prefix  padded   expected
            (   0,    true,  true,     "0o00000011064"),
            (   2,    true,  true,     "0o02215177777"),
            (   4,    true,  true,     "0o37777777777"),
        ];

        for (index, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = IntegerReader::U32(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                OctalFormatter::new_integer(prefix, padded).render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_octal_u64() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  prefix  padded  expected
            (   0,    false,  false,  "443237777777777"),
            (   8,    false,  false,  "1777777777777777777777"),

            // index  prefix  padded  expected
            (   0,    false,  true,   "0000000443237777777777"),
            (   8,    false,  true,   "1777777777777777777777"),

            // index  prefix  padded  expected
            (   0,    true,   false,  "0o443237777777777"),
            (   8,    true,   false,  "0o1777777777777777777777"),

            // index  prefix  padded  expected
            (   0,    true,   true,   "0o0000000443237777777777"),
            (   8,    true,   true,   "0o1777777777777777777777"),

        ];

        for (index, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = IntegerReader::U64(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                OctalFormatter::new_integer(prefix, padded).render(number),
            );
        }

        Ok(())
    }
}
