use num_traits::FromPrimitive;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::generic_number::{GenericNumber, GenericFormatter, GenericFormatterImpl};

mod terraria_game_mode;
pub use terraria_game_mode::TerrariaGameMode;

mod terraria_version;
pub use terraria_version::TerrariaVersion;

mod terraria_item;
pub use terraria_item::TerrariaItem;

mod windows_error;
pub use windows_error::WindowsError;

mod test_enum;
pub use test_enum::TestEnum;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CustomEnumType {
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EnumType {
    TerrariaGameMode,
    TerrariaVersion,
    TerrariaItem,
    WindowsError,
    TestEnum,
    //Custom(CustomEnumType),
}

/// Render a [`GenericNumber`] as an enumeration.
///
/// An enumeration, in this case, is a set of unique named values grouped under
/// an [`EnumType`]. I add different [`EnumType`] values as I need them, they
/// are all under this same module.
///
/// I plan to do custom enums as well, though I'm not sure what that'll look
/// like.
///
/// # Example
///
/// ```
/// use libh2gb::generic_number::*;
///
/// // Create a GenericNumber directly - normally you'd use a GenericReader
/// let number = GenericNumber::from(2);
///
/// // As a Windows error, 2 is file not found
/// assert_eq!("WindowsError::ERROR_FILE_NOT_FOUND (ERROR_FILE_NOT_FOUND)", EnumFormatter::new(EnumType::WindowsError).render(number).unwrap());
///
/// // As a Terraria game mode, 2 is Hardcore
/// assert_eq!("TerrariaGameMode::HardCore (Hard Core)", EnumFormatter::new(EnumType::TerrariaGameMode).render(number).unwrap());
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnumFormatter {
    enum_type: EnumType,
}

impl EnumFormatter {
    pub fn new(enum_type: EnumType) -> GenericFormatter {
        GenericFormatter::Enum(Self {
            enum_type: enum_type,
        })
    }
}

impl GenericFormatterImpl for EnumFormatter {
    fn render(&self, number: GenericNumber) -> SimpleResult<String> {
        // Always treat the number as a 64-bit unsigned (signed values will work
        // correctly)
        let number = if number.can_be_u64() {
            number.as_u64()?
        } else if number.can_be_i64() {
            number.as_i64()? as u64
        } else {
            bail!("The type is not compatible with enumerations (must be an integer with a maximum size of 64 bits");
        };

        let s = match self.enum_type {
            EnumType::TestEnum         =>         TestEnum::from_u64(number).map(|v| format!("{:?} ({})", v, v.to_string())),
            EnumType::TerrariaGameMode => TerrariaGameMode::from_u64(number).map(|v| format!("{:?} ({})", v, v.to_string())),
            EnumType::TerrariaVersion  =>  TerrariaVersion::from_u64(number).map(|v| format!("{:?} ({})", v, v.to_string())),
            EnumType::TerrariaItem     =>     TerrariaItem::from_u64(number).map(|v| format!("{:?} ({})", v, v.to_string())),
            EnumType::WindowsError     =>     WindowsError::from_u64(number).map(|v| format!("{:?} ({})", v, v.to_string())),
        }.unwrap_or(format!("unknown_0x{:016x}", number));

        Ok(format!("{:?}::{}", self.enum_type, s))
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
          // number                                      expected
            (GenericNumber::from(0u32),                  "TestEnum::Zero (Zero)"),
            (GenericNumber::from(0i32),                  "TestEnum::Zero (Zero)"),

            (GenericNumber::from(-1),                    "TestEnum::NegativeOne (NegativeOne)"),
            (GenericNumber::from(0xffffffffffffffffu64), "TestEnum::NegativeOne (NegativeOne)"),

            (GenericNumber::from(1u32),                  "TestEnum::One (One)"),
            (GenericNumber::from(1i32),                  "TestEnum::One (One)"),

            (GenericNumber::from(0xffffffffu32),         "TestEnum::U32Max (U32Max)"),
            (GenericNumber::from(0x00000000ffffffffi64), "TestEnum::U32Max (U32Max)"),

            (GenericNumber::from(0x7fffffffu32),         "TestEnum::I32Max (I32Max)"),
            (GenericNumber::from(0x7fffffffi32),         "TestEnum::I32Max (I32Max)"),

            (GenericNumber::from(0x7fffffffffffffffi64), "TestEnum::I64Max (I64Max)"),
            (GenericNumber::from(0x7fffffffffffffffu64), "TestEnum::I64Max (I64Max)"),
        ];

        for (number, expected) in tests {
            assert_eq!(
                expected,
                EnumFormatter::new(EnumType::TestEnum).render(number)?,
            );
        }

        Ok(())
    }
}
