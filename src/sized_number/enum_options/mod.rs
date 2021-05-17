use serde::{Serialize, Deserialize};

use num_traits::FromPrimitive;

pub mod terraria_game_mode;
pub use terraria_game_mode::TerrariaGameMode;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CustomEnumType {
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EnumType {
    TerrariaGameMode,
    Custom(CustomEnumType),
}

impl EnumType {
    pub fn to_s(self, value: u64) -> String {
        match self {
            Self::TerrariaGameMode => match TerrariaGameMode::from_u64(value) {
                Some(v) => format!("TerrariaGameMode::{:?}", v),
                None => format!("TerrariaGameMode::unknown_0x{:08x}", value),
            },
            Self::Custom(_) => "TODO".to_string(),
        }
    }
}
