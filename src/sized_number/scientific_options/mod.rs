use std::fmt::LowerExp;
use serde::{Serialize, Deserialize};

/// Configure display options for [`SizedDisplay::Scientific`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScientificOptions {
    /// Print the `e` in the scientific notation will be uppercase (`1E0`
    /// instead of `1e0`).
    pub uppercase: bool,
}

impl Default for ScientificOptions {
    fn default() -> Self {
        Self {
            uppercase: false,
        }
    }
}

impl ScientificOptions {
    /// An internal function to help with displaying scientific / exponential
    /// notation.
    pub fn to_s(self, v: Box<dyn LowerExp>) -> String {
        let mut v = format!("{:e}", v.as_ref());

        if self.uppercase {
            v = v.to_uppercase();
        }

        v
    }
}
