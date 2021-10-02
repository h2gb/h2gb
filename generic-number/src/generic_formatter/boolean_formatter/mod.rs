use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::{GenericNumber, GenericFormatter, GenericFormatterImpl, Integer, IntegerRenderer, IntegerRendererImpl};

/// Render a [`GenericNumber`] as a true/false value (1 is true, 0 is false)
///
/// # Example
///
/// ```
/// // TODO: Boolean example
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BooleanFormatter {
}

impl BooleanFormatter {
    pub fn new() -> GenericFormatter {
        GenericFormatter::Boolean(Self { })
    }

    pub fn new_integer() -> IntegerRenderer {
        IntegerRenderer::Boolean(Self { })
    }
}

impl GenericFormatterImpl for BooleanFormatter {
    fn render(&self, number: GenericNumber) -> SimpleResult<String> {
        match number {
            GenericNumber::U8(v)      => Ok(format!("{}", v != 0)),
            GenericNumber::U16(v)     => Ok(format!("{}", v != 0)),
            GenericNumber::U32(v)     => Ok(format!("{}", v != 0)),
            GenericNumber::U64(v)     => Ok(format!("{}", v != 0)),
            GenericNumber::U128(v)    => Ok(format!("{}", v != 0)),

            GenericNumber::I8(v)      => Ok(format!("{}", v != 0)),
            GenericNumber::I16(v)     => Ok(format!("{}", v != 0)),
            GenericNumber::I32(v)     => Ok(format!("{}", v != 0)),
            GenericNumber::I64(v)     => Ok(format!("{}", v != 0)),
            GenericNumber::I128(v)    => Ok(format!("{}", v != 0)),

            GenericNumber::F32(_)     => bail!("Cannot display floating point as a boolean"),
            GenericNumber::F64(_)     => bail!("Cannot display floating point as a boolean"),

            GenericNumber::Char(_, _) => bail!("Cannot display character as a boolean"),
        }
    }
}

impl IntegerRendererImpl for BooleanFormatter {
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
