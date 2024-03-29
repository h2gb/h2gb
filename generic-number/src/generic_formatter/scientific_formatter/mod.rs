use serde::{Serialize, Deserialize};

use crate::{Integer, IntegerRenderer, IntegerRendererTrait, Float, FloatRenderer, FloatRendererTrait};

/// Render a [`Integer`] as a scientific (exponential) value.
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create a Integer directly - normally you'd use a IntegerReader
/// let number = Integer::from(100u64);
///
/// // Default 'pretty' formatter
/// assert_eq!("1e2", ScientificFormatter::pretty_integer().render(number));
///
/// // Also works on floating point
/// let number = Float::from(314.159f32);
/// assert_eq!("3.14159e2", ScientificFormatter::pretty_float().render(number));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScientificFormatter {
    /// Print the `e` in the scientific notation will be uppercase (`1E0`
    /// instead of `1e0`).
    pub uppercase: bool,
}

impl ScientificFormatter {
    pub fn new_integer(uppercase: bool) -> IntegerRenderer {
        IntegerRenderer::Scientific(Self {
            uppercase: uppercase
        })
    }

    pub fn pretty_integer() -> IntegerRenderer {
        Self::new_integer(false)
    }

    pub fn new_float(uppercase: bool) -> FloatRenderer {
        FloatRenderer::Scientific(Self {
            uppercase: uppercase
        })
    }

    pub fn pretty_float() -> FloatRenderer {
        Self::new_float(false)
    }
}

impl IntegerRendererTrait for ScientificFormatter {
    fn render_integer(&self, number: Integer) -> String {
        let rendered = match self.uppercase {
            false => format!("{:e}", number),
            true  => format!("{:E}", number),
        };

        rendered
    }
}

impl FloatRendererTrait for ScientificFormatter {
    fn render_float(&self, number: Float) -> String {
        let rendered = match self.uppercase {
            false => format!("{:e}", number),
            true  => format!("{:E}", number),
        };

        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::{Context, Endian, IntegerReader, FloatReader};

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
            let number = IntegerReader::U32(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                ScientificFormatter::new_integer(uppercase).render(number),
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
            let number = IntegerReader::I32(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                ScientificFormatter::new_integer(uppercase).render(number),
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
            let number = FloatReader::F64(Endian::Big).read(context)?;

            assert_eq!(
                expected,
                ScientificFormatter::new_float(uppercase).render(number),
            );
        }

        Ok(())
    }

}
