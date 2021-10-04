use serde::{Serialize, Deserialize};

use crate::{Float, DefaultFormatter, ScientificFormatter};

/// Define the interface for rendering a [`Float`].
pub trait FloatRendererTrait {
    fn render_float(&self, number: Float) -> String;
}

/// Configure how a [`Float`] is rendered.
///
/// You don't generally want to create this enum directly; instead, use the
/// `new()` or `pretty()` methods in the formatter you want.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FloatRenderer {
    Default(DefaultFormatter),
    Scientific(ScientificFormatter),
}

impl FloatRenderer {
    /// Render the given number
    pub fn render(&self, number: Float) -> String {
        match self {
            Self::Default(f)    => f.render_float(number),
            Self::Scientific(f) => f.render_float(number),
        }
    }
}
