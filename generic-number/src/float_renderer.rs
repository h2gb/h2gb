use serde::{Serialize, Deserialize};

use crate::{Float, DefaultFormatter, ScientificFormatter};

/// Define the interface for rendering a [`Float`].
pub trait FloatRendererTrait {
    fn render_float(&self, number: impl Into<Float>) -> String;
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
    pub fn render_float(&self, number: impl Into<Float>) -> String {
        match self {
            Self::Default(f)    => f.render_float(number),
            Self::Scientific(f) => f.render_float(number),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use crate::Float;

    #[test]
    fn test_pretty() -> SimpleResult<()> {
        assert_eq!("3.14e0", ScientificFormatter::new_pretty().render_float(Float::from(3.14f32)));
        assert_eq!("3.14",   DefaultFormatter::new().render_float(Float::from(3.14f32)));

        assert_eq!("3.14e0", ScientificFormatter::new_pretty().render_float(Float::from(3.14f64)));
        assert_eq!("3.14",   DefaultFormatter::new().render_float(Float::from(3.14f64)));

        Ok(())
    }
}
