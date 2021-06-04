use num_traits::FromPrimitive;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::sized_number::{GenericNumber, SizedOptions, SizedDisplay};

mod terraria_game_mode;
pub use terraria_game_mode::TerrariaGameMode;

mod terraria_version;
pub use terraria_version::TerrariaVersion;

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
    WindowsError,
    TestEnum,
    //Custom(CustomEnumType),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnumOptions {
    enum_type: EnumType,
}

impl EnumOptions {
    pub fn new(enum_type: EnumType) -> SizedDisplay {
        SizedDisplay::Enum(Self {
            enum_type: enum_type,
        })
    }
}

impl SizedOptions for EnumOptions {
    fn to_string(&self, number: GenericNumber) -> SimpleResult<String> {
        let s = if number.can_be_u64() {
            let v = number.as_u64()?;

            match self.enum_type {
                EnumType::TerrariaGameMode => TerrariaGameMode::from_u64(v).map(|v| format!("{:?} ({})", v, v.to_string())),
                EnumType::TerrariaVersion  => TerrariaVersion::from_u64(v).map( |v| format!("{:?} ({})", v, v.to_string())),
                EnumType::TestEnum         => TestEnum::from_u64(v).map(        |v| format!("{:?} ({})", v, v.to_string())),
                EnumType::WindowsError     => WindowsError::from_u64(v).map(    |v| format!("{:?} ({})", v, v.to_string())),
            }.unwrap_or(format!("unknown_0x{:016x}", v))
        } else if number.can_be_i64() {
            let v = number.as_i64()?;

            match self.enum_type {
                EnumType::TerrariaGameMode => TerrariaGameMode::from_i64(v).map(|v| format!("{:?} ({})", v, v.to_string())),
                EnumType::TerrariaVersion  => TerrariaVersion::from_i64(v).map( |v| format!("{:?} ({})", v, v.to_string())),
                EnumType::TestEnum         => TestEnum::from_i64(v).map(        |v| format!("{:?} ({})", v, v.to_string())),
                EnumType::WindowsError     => WindowsError::from_i64(v).map(    |v| format!("{:?} ({})", v, v.to_string())),
            }.unwrap_or(format!("unknown_0x{:016x}", v))
        } else {
            bail!("The type is not compatible with enumerations (must be an integer with a maximum size of 64 bits");
        };

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
          // number                              expected
            (GenericNumber::from(0u32),          "TestEnum::Zero (Zero)"     ),
            (GenericNumber::from(0xffffffffu32), "TestEnum::U32Max (U32Max)" ),
        ];

        for (number, expected) in tests {
            assert_eq!(
                expected,
                EnumOptions::new(EnumType::TestEnum).to_string(number)?,
            );
        }

        Ok(())
    }
}
