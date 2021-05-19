use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::sized_number::SizedOptions;

/// Configure display options for [`SizedDisplay::Decimal`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DecimalOptions {
}

impl Default for DecimalOptions {
    fn default() -> Self {
        Self {
        }
    }
}

impl SizedOptions for DecimalOptions {
    fn to_s_i8(&self, v:   i8)   -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_i16(&self, v:  i16)  -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_i32(&self, v:  i32)  -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_i64(&self, v:  i64)  -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_i128(&self, v: i128) -> SimpleResult<String> {
        Ok(format!("{}", v))
    }


    fn to_s_u8(&self, v:   u8)   -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_u16(&self, v:  u16)  -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_u32(&self, v:  u32)  -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_u64(&self, v:  u64)  -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_u128(&self, v: u128) -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_f32(&self, v:  f32) -> SimpleResult<String> {
        Ok(format!("{}", v))
    }

    fn to_s_f64(&self, v:  f64) -> SimpleResult<String> {
        Ok(format!("{}", v))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::sized_number::{Context, Endian, SizedDisplay, SizedDefinition};

    #[test]
    fn test_decimal_u8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   1,    "127"),
            (   2,    "128"),
            (   3,    "255"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U8.to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_i8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   1,    "127"),
            (   2,    "-128"),
            (   3,    "-1"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I8.to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_u16() -> SimpleResult<()> {
        let data = b"\x00\xFF\x00\x01\x00\x00\xFF\xFF".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "255"),
            (   2,    "1"),
            (   4,    "0"),
            (   6,    "65535"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U16(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_u32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\xff\xff\xff\xff\x7f\xff\xff\xff\x80\x00\x00\x00".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   4,    "4294967295"),
            (   8,    "2147483647"),
            (  12,    "2147483648"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_i32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\xff\xff\xff\xff\x7f\xff\xff\xff\x80\x00\x00\x00".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   4,    "-1"),
            (   8,    "2147483647"),
            (  12,    "-2147483648"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_i64() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x7f\xff\xff\xff\xff\xff\xff\xff\x80\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff".to_vec();

        let tests = vec![
            // index  expected
            (  0,    "0"),
            (  8,    "9223372036854775807"),
            (  16,   "-9223372036854775808"),
            (  24,   "-1"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_u128() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  expected
            (  0,    "0"),
            (  16,   "340282366920938463463374607431768211455"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U128(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_i128() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  expected
            (  0,    "0"),
            (  16,   "-1"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I128(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_f32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\xff\xff\xff\xff\x41\xc8\x00\x00\x40\x48\xf5\xc3".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   4,    "NaN"),
            (   8,    "25"), // From https://en.wikipedia.org/wiki/Single-precision_floating-point_format#Converting_from_single-precision_binary_to_decimal
            (  12,    "3.14"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::F32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_f64_big_endian() -> SimpleResult<()> {
        // I wrote and disassembled a simple C program to get these strings.. double is hard
        let data = b"\x40\x09\x1e\xb8\x51\xeb\x85\x1f\x40\x09\x33\x33\x33\x33\x33\x33".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "3.14"),
            (   8,    "3.15"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::F64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_f64_little_endian() -> SimpleResult<()> {
        // I wrote and disassembled a simple C program to get these strings.. double is hard
        let data = b"\x1F\x85\xEB\x51\xB8\x1E\x09\x40\x33\x33\x33\x33\x33\x33\x09\x40".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "3.14"),
            (   8,    "3.15"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::F64(Endian::Little).to_string(
                    context,
                    SizedDisplay::Decimal(Default::default())
                )?
            );
        }

        Ok(())
    }
}
