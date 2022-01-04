use serde::{Serialize, Deserialize};

use crate::{Integer, IntegerRenderer, IntegerRendererTrait, Float, FloatRenderer, FloatRendererTrait, Character, CharacterRenderer, CharacterRendererTrait};

/// Render a [`Integer`], [`Character`], or [`Float`] as whatever the default is.
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create a Integer directly - normally you'd use a IntegerReader
/// let number = Integer::from(1234u32);
///
/// // DefaultFormatter has no special options
/// assert_eq!("1234", DefaultFormatter::new_integer().render(number));
///
/// // Also handles signed values correctly, using the Integer's type
/// let number = Integer::from(-1234i32);
/// assert_eq!("-1234", DefaultFormatter::new_integer().render(number));
///
/// // Handles floating point correctly, as well
/// let number = Float::from(314.159f32);
/// assert_eq!("314.159", DefaultFormatter::new_float().render(number));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DefaultFormatter {
}

impl DefaultFormatter {
    pub fn new_integer() -> IntegerRenderer {
        IntegerRenderer::Default(Self { })
    }

    pub fn new_float() -> FloatRenderer {
        FloatRenderer::Default(Self { })
    }

    pub fn new_character() -> CharacterRenderer {
        CharacterRenderer::Default(Self { })
    }
}

impl IntegerRendererTrait for DefaultFormatter {
    fn render_integer(&self, number: Integer) -> String {
        format!("{}", number)
    }
}

impl FloatRendererTrait for DefaultFormatter {
    fn render_float(&self, number: Float) -> String {
        format!("{}", number)
    }
}

impl CharacterRendererTrait for DefaultFormatter {
    fn render_character(&self, number: Character) -> String {
        format!("{}", number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::{Context, Endian, IntegerReader, FloatReader};

    #[test]
    fn test_default_u8() -> SimpleResult<()> {
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
            let number = IntegerReader::U8.read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_integer().render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_i8() -> SimpleResult<()> {
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
            let number = IntegerReader::I8.read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_integer().render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_u16() -> SimpleResult<()> {
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
            let number = IntegerReader::U16(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_integer().render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_u32() -> SimpleResult<()> {
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
            let number = IntegerReader::U32(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_integer().render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_i32() -> SimpleResult<()> {
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
            let number = IntegerReader::I32(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_integer().render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_i64() -> SimpleResult<()> {
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
            let number = IntegerReader::I64(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_integer().render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_u128() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  expected
            (  0,    "0"),
            (  16,   "340282366920938463463374607431768211455"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = IntegerReader::U128(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_integer().render(number),
            );
        }

        Ok(())
    }

    // #[test]
    // fn test_default_i128() -> SimpleResult<()> {
    //     let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec();

    //     let tests = vec![
    //         // index  expected
    //         (  0,    "0"),
    //         (  16,   "-1"),
    //     ];

    //     for (index, expected) in tests {
    //         let context = Context::new_at(&data, index);
    //         let number = IntegerReader::I128(Endian::Big).read(context)?;

    //         assert_eq!(
    //             expected,
    //             DefaultFormatter::new_integer().render(number),
    //         );
    //     }

    //     Ok(())
    // }

    #[test]
    fn test_default_f32() -> SimpleResult<()> {
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
            let number = FloatReader::F32(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_float().render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_f64_big_endian() -> SimpleResult<()> {
        // I wrote and disassembled a simple C program to get these strings.. double is hard
        let data = b"\x40\x09\x1e\xb8\x51\xeb\x85\x1f\x40\x09\x33\x33\x33\x33\x33\x33".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "3.14"),
            (   8,    "3.15"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = FloatReader::F64(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_float().render(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_f64_little_endian() -> SimpleResult<()> {
        // I wrote and disassembled a simple C program to get these strings.. double is hard
        let data = b"\x1F\x85\xEB\x51\xB8\x1E\x09\x40\x33\x33\x33\x33\x33\x33\x09\x40".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "3.14"),
            (   8,    "3.15"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = FloatReader::F64(Endian::Little).read(context)?;

            assert_eq!(
                expected,
                DefaultFormatter::new_float().render(number),
            );
        }

        Ok(())
    }
}
