use serde::{Serialize, Deserialize};

use num_traits::FromPrimitive;

pub mod terraria_game_mode;
pub use terraria_game_mode::TerrariaGameMode;

pub mod test_enum;
pub use test_enum::TestEnum;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CustomEnumType {
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EnumType {
    TerrariaGameMode,
    TestEnum,
    //Custom(CustomEnumType),
}

impl EnumType {
    pub fn u64_to_s(self, value: u64) -> String {
        match self {
            Self::TerrariaGameMode => match TerrariaGameMode::from_u64(value) {
                Some(v) => format!("TerrariaGameMode::{:?}", v),
                None => format!("TerrariaGameMode::unknown_0x{:016x}", value),
            },

            Self::TestEnum => match TestEnum::from_u64(value) {
                Some(v) => format!("TestEnum::{:?}", v),
                None => format!("TestEnum::unknown_0x{:016x}", value),
            },

            //Self::Custom(_) => "TODO".to_string(),
        }
    }

    pub fn i64_to_s(self, value: i64) -> String {
        match self {
            Self::TerrariaGameMode => match TerrariaGameMode::from_i64(value) {
                Some(v) => format!("TerrariaGameMode::{:?}", v),
                None => format!("TerrariaGameMode::unknown_0x{:016x}", value),
            },

            Self::TestEnum => match TestEnum::from_i64(value) {
                Some(v) => format!("TestEnum::{:?}", v),
                None => format!("TestEnum::unknown_0x{:016x}", value),
            },
            //Self::Custom(_) => "TODO".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::sized_number::{SizedDefinition, SizedDisplay, Context, Endian};

    #[test]
    fn test_enum_zero() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00";

        let s = SizedDefinition::U32(Endian::Big).to_string(
            Context::new_at(&data.to_vec(), 0),
            SizedDisplay::Enum(EnumType::TestEnum),
        )?;
        assert_eq!("TestEnum::Zero", s);

        Ok(())
    }

    #[test]
    fn test_enum_max() -> SimpleResult<()> {
        let data = b"\xff\xff\xff\xff";

        // u32 is unsigned, so it'll count as 0x00000000FFFFFFFF
        let s = SizedDefinition::U32(Endian::Big).to_string(
            Context::new_at(&data.to_vec(), 0),
            SizedDisplay::Enum(EnumType::TestEnum),
        )?;
        assert_eq!("TestEnum::U32Max", s);

        // i32 is signed, so it'll count as 0xFFFFFFFFFFFFFFFF
        let s = SizedDefinition::I32(Endian::Big).to_string(
            Context::new_at(&data.to_vec(), 0),
            SizedDisplay::Enum(EnumType::TestEnum),
        )?;
        assert_eq!("TestEnum::NegativeOne", s);

        Ok(())
    }
}
