use serde::{Serialize, Deserialize};
use num_derive::FromPrimitive;
use std::fmt;

#[derive(FromPrimitive, Debug, Clone, Copy, Serialize, Deserialize)]

pub enum TerrariaGameMode {
  Classic     = 0,
  MediumCore  = 1,
  HardCore    = 2,
  JourneyMode = 3,
}

impl fmt::Display for TerrariaGameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match  *self {
            Self::Classic        => write!(f, "Classic"),
            Self::MediumCore     => write!(f, "Medium Core"),
            Self::HardCore       => write!(f, "Hard Core"),
            Self::JourneyMode    => write!(f, "Journey Mode"),
        }
    }
}
