use serde::{Serialize, Deserialize};

use crate::{Integer, BinaryFormatter, BooleanFormatter, DefaultFormatter, HexFormatter, OctalFormatter, ScientificFormatter};

// Define the interface for rendering an integer
pub trait IntegerFormatterImpl {
    fn render_integer(&self, number: Integer) -> String;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntegerFormatter {
    Binary(BinaryFormatter),
    Boolean(BooleanFormatter),
    Default(DefaultFormatter),
    Hex(HexFormatter),
    Octal(OctalFormatter),
    Scientific(ScientificFormatter),
}

impl IntegerFormatter {
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

        let formatter = DefaultFormatter::integer();
        let integer = IntegerReader::U8.read(Context::new_at(&data, 0))?;

        assert_eq!("0".to_string(), formatter.render(integer));

        Ok(())
    }
}
