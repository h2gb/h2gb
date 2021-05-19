use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::sized_number::SizedOptions;

/// Configure display options for [`SizedDisplay::Scientific`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScientificOptions {
    /// Print the `e` in the scientific notation will be uppercase (`1E0`
    /// instead of `1e0`).
    pub uppercase: bool,
}

impl Default for ScientificOptions {
    fn default() -> Self {
        Self {
            uppercase: false,
        }
    }
}

impl SizedOptions for ScientificOptions {
    fn to_s_i8(&self, v:   i8)   -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_i16(&self, v:  i16)  -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_i32(&self, v:  i32)  -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_i64(&self, v:  i64)  -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_i128(&self, v: i128) -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }


    fn to_s_u8(&self, v:   u8)   -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_u16(&self, v:  u16)  -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_u32(&self, v:  u32)  -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_u64(&self, v:  u64)  -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_u128(&self, v: u128) -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_f32(&self, v:  f32) -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }

    fn to_s_f64(&self, v:  f64) -> SimpleResult<String> {
        Ok(match self.uppercase {
            false => format!("{:e}", v),
            true  => format!("{:E}", v),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::sized_number::{Context, Endian, SizedDisplay, SizedDefinition};

    #[test]
    fn test_scientific_u32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x7f\xff\xff\xff\x80\x00\x00\x00\xff\xff\xff\xff".to_vec();

        let tests = vec![
            // index  uppercase  expected
            (   0,    false,     "0e0"),
            (   4,    false,     "2.147483647e9"),
            (   8,    false,     "2.147483648e9"),
            (  12,    false,     "4.294967295e9"),
            (   0,    true,      "0E0"),
            (   4,    true,      "2.147483647E9"),
            (   8,    true,      "2.147483648E9"),
            (  12,    true,      "4.294967295E9"),
        ];

        for (index, uppercase, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Scientific(ScientificOptions {
                        uppercase: uppercase,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_scientific_i32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x7f\xff\xff\xff\x80\x00\x00\x00\xff\xff\xff\xff".to_vec();

        let tests = vec![
            // index  uppercase  expected
            (   0,    false,     "0e0"),
            (   4,    false,     "2.147483647e9"),
            (   8,    false,     "-2.147483648e9"),
            (  12,    false,     "-1e0"),
            (   0,    true,      "0E0"),
            (   4,    true,      "2.147483647E9"),
            (   8,    true,      "-2.147483648E9"),
            (  12,    true,      "-1E0"),
        ];

        for (index, uppercase, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Scientific(ScientificOptions {
                        uppercase: uppercase,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_exponent_f64() -> SimpleResult<()> {
        // I wrote and disassembled a simple C program to get these strings.. double is hard
        let data = b"\x40\x09\x1e\xb8\x51\xeb\x85\x1f\x40\x09\x33\x33\x33\x33\x33\x33".to_vec();

        let tests = vec![
            // index  uppercase expected
            (   0,    false,    "3.14e0"),
            (   8,    false,    "3.15e0"),
            (   0,    true,     "3.14E0"),
            (   8,    true,     "3.15E0"),
        ];

        for (index, uppercase, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::F64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Scientific(ScientificOptions {
                        uppercase: uppercase,
                    }),
                )?
            );
        }

        Ok(())
    }

}
