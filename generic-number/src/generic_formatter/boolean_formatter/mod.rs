use serde::{Serialize, Deserialize};

use crate::{Integer, IntegerRenderer, IntegerRendererTrait};

/// Render a [`GenericNumber`] as a true/false value (1 is true, 0 is false)
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create an Integer directly - normally you'd use a IntegerReader
/// let number = Integer::from(1);
/// assert_eq!("true", BooleanFormatter::new_integer().render(number));
///
/// let number = Integer::from(0);
/// assert_eq!("false", BooleanFormatter::new_integer().render(number));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BooleanFormatter {
}

impl BooleanFormatter {
    pub fn new_integer() -> IntegerRenderer {
        IntegerRenderer::Boolean(Self { })
    }
}

impl IntegerRendererTrait for BooleanFormatter {
    fn render_integer(&self, number: Integer) -> String {
        // Ironically, boolean is both the simplest (conceptually), and the
        // hardest to program since we can't really compare integers of a
        // different size to 0
        match number {
            Integer::U8(v)      => format!("{}", v != 0),
            Integer::U16(v)     => format!("{}", v != 0),
            Integer::U32(v)     => format!("{}", v != 0),
            Integer::U64(v)     => format!("{}", v != 0),
            Integer::U128(v)    => format!("{}", v != 0),

            Integer::I8(v)      => format!("{}", v != 0),
            Integer::I16(v)     => format!("{}", v != 0),
            Integer::I32(v)     => format!("{}", v != 0),
            Integer::I64(v)     => format!("{}", v != 0),
            Integer::I128(v)    => format!("{}", v != 0),
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
                BooleanFormatter::new_integer().render(number),
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
                BooleanFormatter::new_integer().render(number),
            );
        }

        Ok(())
    }
}
