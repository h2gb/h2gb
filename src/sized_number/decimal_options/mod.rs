use std::fmt::Display;
use serde::{Serialize, Deserialize};

/// Configure display options for [`SizedDisplay::Decimal`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DecimalOptions {
}

impl Default for DecimalOptions {
    fn default() -> Self {
        Self {
        }
    }
}

impl DecimalOptions {
    /// An internal function to help with displaying octal
    pub fn to_s(self, v: Box<dyn Display>) -> String {
        format!("{}", v.as_ref())
    }
}
