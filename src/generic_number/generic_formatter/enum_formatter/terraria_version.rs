use serde::{Serialize, Deserialize};
use num_derive::FromPrimitive;
use std::fmt;

#[derive(FromPrimitive, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TerrariaVersion {
    Version1_4_0_1 = 230,
    Version1_4_1_2 = 234,
    Version1_4_2   = 235,
    Version1_4_2_1 = 236,
    Version1_4_2_2 = 237,
    Version1_4_2_3 = 238,
}

impl fmt::Display for TerrariaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match  *self {
            Self::Version1_4_0_1 => write!(f, "1.4.0.1"),
            Self::Version1_4_1_2 => write!(f, "1.4.2.1"),
            Self::Version1_4_2   => write!(f, "1.4.2"),
            Self::Version1_4_2_1 => write!(f, "1.4.2.1"),
            Self::Version1_4_2_2 => write!(f, "1.4.2.2"),
            Self::Version1_4_2_3 => write!(f, "1.4.2.3"),
        }
    }
}
