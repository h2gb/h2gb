use serde::{Serialize, Deserialize};

use crate::{Integer, BinaryFormatter, BooleanFormatter, DefaultFormatter, HexFormatter, OctalFormatter, ScientificFormatter};

/// Define the interface for rendering an integer
pub trait IntegerRendererImpl {
    fn render_integer(&self, number: Integer) -> String;
}

/// Configure how a [`Float`] is rendered.
///
/// You don't generally want to create this enum directly; instead, use the
/// `new()` or `pretty()` methods in the formatter you want.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntegerRenderer {
    Binary(BinaryFormatter),
    Boolean(BooleanFormatter),
    Default(DefaultFormatter),
    Hex(HexFormatter),
    Octal(OctalFormatter),
    Scientific(ScientificFormatter),
}

impl IntegerRenderer {
    pub fn render(self, v: Integer) -> String {
        match self {
            Self::Binary(f)     => f.render_integer(v),
            Self::Boolean(f)    => f.render_integer(v),
            Self::Default(f)    => f.render_integer(v),
            Self::Hex(f)        => f.render_integer(v),
            Self::Octal(f)      => f.render_integer(v),
            Self::Scientific(f) => f.render_integer(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    use crate::{Context, IntegerReader, DefaultFormatter};

    #[test]
    fn test_render() -> SimpleResult<()> {
        let data = b"\x00\x01\x02\x03".to_vec();

        let formatter = DefaultFormatter::new_integer();
        let integer = IntegerReader::U8.read(Context::new_at(&data, 0))?;

        assert_eq!("0".to_string(), formatter.render(integer));

        Ok(())
    }
}
