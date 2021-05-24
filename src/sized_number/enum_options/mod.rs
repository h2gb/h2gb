use num_traits::FromPrimitive;
use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::sized_number::SizedOptions;

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

impl SizedOptions for EnumType {
    fn to_s_i8(&self, v:   i8)   -> SimpleResult<String> {
        self.to_s_i64(v as i64)
    }

    fn to_s_i16(&self, v:  i16)  -> SimpleResult<String> {
        self.to_s_i64(v as i64)
    }

    fn to_s_i32(&self, v:  i32)  -> SimpleResult<String> {
        self.to_s_i64(v as i64)
    }

    fn to_s_i64(&self, v:  i64)  -> SimpleResult<String> {
        // Get the name of the option
        let s: String = match self {
            Self::TerrariaGameMode => TerrariaGameMode::from_i64(v).map(|v| format!("{:?} ({})", v, v.to_string())),
            Self::TerrariaVersion  => TerrariaVersion::from_i64(v).map( |v| format!("{:?} ({})", v, v.to_string())),
            Self::TestEnum         => TestEnum::from_i64(v).map(        |v| format!("{:?} ({})", v, v.to_string())),
            Self::WindowsError     => WindowsError::from_i64(v).map(    |v| format!("{:?} ({})", v, v.to_string())),
        }.unwrap_or(format!("unknown_0x{:016x}", v));

        // Prefix it with the enum name
        Ok(format!("{:?}::{}", self, s))
    }

    fn to_s_i128(&self, _v: i128) -> SimpleResult<String> {
        bail!("128-bit values cannot be displayed as an enum")
    }

    fn to_s_u8(&self, v:   u8)   -> SimpleResult<String> {
        self.to_s_u64(v as u64)
    }

    fn to_s_u16(&self, v:  u16)  -> SimpleResult<String> {
        self.to_s_u64(v as u64)
    }

    fn to_s_u32(&self, v:  u32)  -> SimpleResult<String> {
        self.to_s_u64(v as u64)
    }

    fn to_s_u64(&self, v:  u64)  -> SimpleResult<String> {
        // Get the name of the option
        let s: String = match self {
            Self::TerrariaGameMode => TerrariaGameMode::from_u64(v).map(|v| format!("{:?} ({})", v, v.to_string())),
            Self::TerrariaVersion  => TerrariaVersion::from_u64(v).map( |v| format!("{:?} ({})", v, v.to_string())),
            Self::WindowsError     => WindowsError::from_u64(v).map(    |v| format!("{:?} ({})", v, v.to_string())),
            Self::TestEnum         => TestEnum::from_u64(v).map(        |v| format!("{:?} ({})", v, v.to_string())),
        }.unwrap_or(format!("unknown_0x{:016x}", v));

        // Prefix it with the enum name
        Ok(format!("{:?}::{}", self, s))
    }

    fn to_s_u128(&self, _v: u128) -> SimpleResult<String> {
        bail!("128-bit values cannot be displayed as an enum")
    }

    fn to_s_f32(&self, _v:  f32) -> SimpleResult<String> {
        bail!("Floating point values cannot be displayed as an enum")
    }

    fn to_s_f64(&self, _v:  f64) -> SimpleResult<String> {
        bail!("Floating point values cannot be displayed as an enum")
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
        assert_eq!("TestEnum::Zero (Zero)", s);

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
        assert_eq!("TestEnum::U32Max (U32Max)", s);

        // i32 is signed, so it'll count as 0xFFFFFFFFFFFFFFFF
        let s = SizedDefinition::I32(Endian::Big).to_string(
            Context::new_at(&data.to_vec(), 0),
            SizedDisplay::Enum(EnumType::TestEnum),
        )?;
        assert_eq!("TestEnum::NegativeOne (NegativeOne)", s);

        Ok(())
    }
}
