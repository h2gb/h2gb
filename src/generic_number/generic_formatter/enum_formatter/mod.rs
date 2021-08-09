use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::data::{enum_exists, from_enum};
use crate::generic_number::{GenericNumber, GenericFormatter, GenericFormatterImpl};

/// Render a [`GenericNumber`] as an enumeration.
///
/// # Example
///
/// ```
/// // TODO: Example of an enumeration
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumFormatter {
    enum_type: String,
}

impl EnumFormatter {
    pub fn new(enum_type: &str) -> SimpleResult<GenericFormatter> {
        // Make sure the enum type exists
        if !enum_exists(enum_type) {
            bail!("No such Enum: {}", enum_type);
        }

        Ok(GenericFormatter::Enum(Self {
            enum_type: enum_type.to_string(),
        }))
    }
}

impl GenericFormatterImpl for EnumFormatter {
    fn render(&self, number: GenericNumber) -> SimpleResult<String> {
        let as_u64 = number.as_u64()?;
        let as_str = match from_enum(&self.enum_type, as_u64)? {
            Some(s) => s.to_string(),
            None    => format!("unknown_0x{:016x}", as_u64),
        };

        Ok(format!("{}::{}", self.enum_type, as_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    #[test]
    fn test_enum() -> SimpleResult<()> {
        let tests = vec![
          // number                     expected
            (GenericNumber::from(0u32), "TerrariaGameMode::Classic"),
        ];

        for (number, expected) in tests {
            assert_eq!(
                expected,
                EnumFormatter::new("TerrariaGameMode")?.render(number)?,
            );
        }

        Ok(())
    }
}
