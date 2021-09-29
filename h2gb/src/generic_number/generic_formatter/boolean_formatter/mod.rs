use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::generic_number::{GenericNumber, GenericFormatter, GenericFormatterImpl};

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
}

impl Default for BooleanFormatter {
    fn default() -> Self {
        Self {
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::generic_number::{Context, GenericReader};

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
            let number = GenericReader::U8.read(context)?;

            assert_eq!(
                expected,
                BooleanFormatter::new().render(number)?,
            );
        }

        Ok(())
    }
}
