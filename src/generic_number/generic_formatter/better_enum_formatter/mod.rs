use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::data::ENUMS;
use crate::generic_number::{GenericNumber, GenericFormatter, GenericFormatterImpl};

/// Render a [`GenericNumber`] as an enumeration.
///
/// # Example
///
/// ```
/// // TODO: Example of an enumeration
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetterEnumFormatter {
    enum_type: String,
}

impl BetterEnumFormatter {
    pub fn new(enum_type: &str) -> SimpleResult<GenericFormatter> {
        // Make sure the enum type exists
        if !ENUMS.contains_key(enum_type) {
            bail!("No such Enum: {}", enum_type);
        }

        Ok(GenericFormatter::BetterEnum(Self {
            enum_type: enum_type.to_string(),
        }))
    }
}

impl GenericFormatterImpl for BetterEnumFormatter {
    fn render(&self, number: GenericNumber) -> SimpleResult<String> {
        let as_u64 = number.as_u64()?;
        let as_str = match ENUMS.get(&self.enum_type).unwrap().get(&as_u64) { // XXX unwrap
            Some(s) => s.clone(),
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
    fn test_better_enum() -> SimpleResult<()> {
        let tests = vec![
          // number                                      expected
            (GenericNumber::from(0u32),                  "TerrariaGameMode::Classic"),
        ];

        for (number, expected) in tests {
            assert_eq!(
                expected,
                BetterEnumFormatter::new("TerrariaGameMode")?.render(number)?,
            );
        }

        Ok(())
    }
}
