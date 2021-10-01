use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::Integer;

/// A trait to simplify rendering.
///
/// I'm not convinced this is actually useful (yet), but it's mostly hidden
/// so I think it's safe to leave it.
pub trait IntegerFormatterImpl {
    fn render(&self, number: Integer) -> String;
}

/// Configure how a [`GenericNumber`] is rendered.
///
/// You don't generally want to create this enum directly; instead, use the
/// `new()` or `pretty()` methods in the formatter you want.
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create a GenericNumber directly - normally you'd read from a buffer
/// let number = GenericNumber::from(1234u32);
///
/// // Create a formatter
/// let formatter = DefaultFormatter::new();
///
/// // Render the number
/// assert_eq!("1234", formatter.render(number).unwrap());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegerFormatter {
    Default(DefaultFormatter),

    Hex(HexFormatter),
    Octal(OctalFormatter),
    Binary(BinaryFormatter),
    Scientific(ScientificFormatter),
    Boolean(BooleanFormatter),
}

impl IntegerFormatter {
    /// Render the given number
    pub fn render(&self, number: Integer) -> String {
        match self {
            Self::Default(o)    => o.render(number),
            Self::Hex(o)        => o.render(number),
            Self::Octal(o)      => o.render(number),
            Self::Binary(o)     => o.render(number),
            Self::Scientific(o) => o.render(number),
            Self::Boolean(o)    => o.render(number),
        }
    }
}
