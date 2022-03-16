use serde::{Serialize, Deserialize};

use crate::{Integer, IntegerRenderer, IntegerRendererTrait};

/// Render a [`Integer`] as "false" (if 0) or "true" (otherwise).
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// assert_eq!("true", BooleanFormatter::new().render_integer(1));
/// assert_eq!("false", BooleanFormatter::new().render_integer(0));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BooleanFormatter {
}

impl From<BooleanFormatter> for IntegerRenderer {
    fn from(f: BooleanFormatter) -> IntegerRenderer {
        IntegerRenderer::Boolean(f)
    }
}

impl BooleanFormatter {
    pub fn new() -> Self {
        Self { }
    }
}

impl IntegerRendererTrait for BooleanFormatter {
    fn render_integer(&self, number: impl Into<Integer>) -> String {
        let number: Integer = number.into();

        // Ironically, boolean is both the simplest (conceptually), and the
        // hardest to program since we can't really compare integers of a
        // different size to 0
        match number {
            Integer::U8(v)      => format!("{}", v != 0),
            Integer::U16(v)     => format!("{}", v != 0),
            Integer::U24(msb,lsb) => format!("{}", msb != 0 && lsb != 0),
            Integer::U32(v)     => format!("{}", v != 0),
            Integer::U64(v)     => format!("{}", v != 0),
            Integer::U128(v)    => format!("{}", v != 0),
            Integer::USize(v)   => format!("{}", v != 0),

            Integer::I8(v)      => format!("{}", v != 0),
            Integer::I16(v)     => format!("{}", v != 0),
            Integer::I32(v)     => format!("{}", v != 0),
            Integer::I64(v)     => format!("{}", v != 0),
            Integer::I128(v)    => format!("{}", v != 0),
            Integer::ISize(v)   => format!("{}", v != 0),
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
    fn test_default_u8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "false"),
            (   1,    "true"),
            (   2,    "true"),
            (   3,    "true"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = IntegerReader::U8.read(context)?;

            assert_eq!(
                expected,
                BooleanFormatter::new().render_integer(number),
            );
        }

        Ok(())
    }

    #[test]
    fn test_default_i8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "false"),
            (   1,    "true"),
            (   2,    "true"),
            (   3,    "true"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);
            let number = IntegerReader::I8.read(context)?;

            assert_eq!(
                expected,
                BooleanFormatter::new().render_integer(number),
            );
        }

        Ok(())
    }
}
