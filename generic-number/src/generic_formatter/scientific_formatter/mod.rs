use serde::{Serialize, Deserialize};

use crate::{Integer, IntegerRenderer, IntegerRendererTrait, Float, FloatRenderer, FloatRendererTrait};

/// Render a [`Integer`] as a scientific (exponential) value.
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Default 'pretty' formatter
/// assert_eq!("1e2", ScientificFormatter::new_pretty().render_integer(100u64));
///
/// // Also works on floating point
/// assert_eq!("3.14159e2", ScientificFormatter::new_pretty().render_float(314.159f32));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScientificFormatter {
    /// Print the `e` in the scientific notation will be uppercase (`1E0`
    /// instead of `1e0`).
    pub uppercase: bool,
}

impl Default for ScientificFormatter {
    fn default() -> Self {
        Self::new_pretty()
    }
}

impl From<ScientificFormatter> for IntegerRenderer {
    fn from(f: ScientificFormatter) -> IntegerRenderer {
        IntegerRenderer::Scientific(f)
    }
}

impl From<ScientificFormatter> for FloatRenderer {
    fn from(f: ScientificFormatter) -> FloatRenderer {
        FloatRenderer::Scientific(f)
    }
}

impl ScientificFormatter {
    pub fn new(uppercase: bool) -> Self {
        Self {
            uppercase: uppercase
        }
    }

    pub fn new_pretty() -> Self {
        Self::new(false)
    }
}

impl IntegerRendererTrait for ScientificFormatter {
    fn render_integer(&self, number: impl Into<Integer>) -> String {
        let number: Integer = number.into();

        let rendered = match self.uppercase {
            false => format!("{:e}", number),
            true  => format!("{:E}", number),
        };

        rendered
    }
}

impl FloatRendererTrait for ScientificFormatter {
    fn render_float(&self, number: impl Into<Float>) -> String {
        let number: Float = number.into();

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
                ScientificFormatter::new(uppercase).render_integer(number),
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
                ScientificFormatter::new(uppercase).render_integer(number),
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
                ScientificFormatter::new(uppercase).render_float(number),
            );
        }

        Ok(())
    }

}
