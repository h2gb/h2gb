use std::fmt::Octal;
use std::mem;

use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::sized_number::SizedOptions;

/// Configure display options for [`SizedDisplay::Octal`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OctalOptions {
    /// Prefix octal strings with `0o`
    pub prefix: bool,

    /// Zero-pad octal strings to the full width - `0001` vs `1`)
    pub padded: bool,
}

impl Default for OctalOptions {
    fn default() -> Self {
        Self {
            prefix: true,
            padded: false,
        }
    }
}

impl OctalOptions {
    /// An internal function to help with displaying octal
    pub fn to_s(self, v: Box<dyn Octal>) -> String {
        let v = v.as_ref();

        if self.padded {
            match (self.prefix, mem::size_of_val(v)) {
                (false, 1)  => format!("{:03o}", v),
                (false, 2)  => format!("{:06o}", v),
                (false, 4)  => format!("{:011o}", v),
                (false, 8)  => format!("{:022o}", v),
                (false, 16) => format!("{:043o}", v),
                (false, _) => format!("{:o}", v),

                (true,  1)  => format!("0o{:03o}", v),
                (true,  2)  => format!("0o{:06o}", v),
                (true,  4)  => format!("0o{:011o}", v),
                (true,  8)  => format!("0o{:022o}", v),
                (true,  16) => format!("0o{:043o}", v),

                (true,   _) => format!("0o{:o}", v),
            }
        } else {
            match self.prefix {
                false => format!("{:o}", v),
                true  => format!("0o{:o}", v),
            }
        }
    }
}

impl SizedOptions for OctalOptions {
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
        bail!("Floating point values cannot be displayed as octal")
    }

    fn to_s_f64(&self, _v:  f64) -> SimpleResult<String> {
        bail!("Floating point values cannot be displayed as octal")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::sized_number::{Context, Endian, SizedDisplay, SizedDefinition};

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

            assert_eq!(
                expected,
                SizedDefinition::U8.to_string(
                    context,
                    SizedDisplay::Octal(OctalOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
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

            assert_eq!(
                expected,
                SizedDefinition::U16(Endian::Big).to_string(
                    context,
                    SizedDisplay::Octal(OctalOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
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

            assert_eq!(
                expected,
                SizedDefinition::U32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Octal(OctalOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
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

            assert_eq!(
                expected,
                SizedDefinition::U64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Octal(OctalOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }
}
