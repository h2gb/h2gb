use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::generic_number::GenericNumber;

mod binary_formatter;
pub use binary_formatter::*;

mod default_formatter;
pub use default_formatter::*;

mod enum_formatter;
pub use enum_formatter::*;

mod hex_formatter;
pub use hex_formatter::*;

mod octal_formatter;
pub use octal_formatter::*;

mod scientific_formatter;
pub use scientific_formatter::*;

/// A trait to simplify rendering.
///
/// I'm not convinced this is actually useful (yet), but it's mostly hidden
/// so I think it's safe to leave it.
pub trait GenericFormatterImpl {
    fn render(&self, number: GenericNumber) -> SimpleResult<String>;
}

/// Configure how a [`GenericNumber`] is rendered.
///
/// You don't generally want to create this enum directly; instead, use the
/// `new()` or `pretty()` methods in the formatter you want.
///
/// # Example
///
/// ```
/// use libh2gb::generic_number::*;
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GenericFormatter {
    Hex(HexFormatter),
    Default(DefaultFormatter),
    Octal(OctalFormatter),
    Binary(BinaryFormatter),
    Scientific(ScientificFormatter),
    Enum(EnumFormatter),
}

impl GenericFormatter {
    /// Internal function to get as the [`GenericFormatterImpl`] trait.
    fn to_formatter(&self) -> Box<dyn GenericFormatterImpl> {
        match self {
            Self::Binary(o)     => Box::new(*o),
            Self::Default(o)    => Box::new(*o),
            Self::Enum(o)       => Box::new(*o),
            Self::Hex(o)        => Box::new(*o),
            Self::Octal(o)      => Box::new(*o),
            Self::Scientific(o) => Box::new(*o),
        }
    }

    /// Attempt to render the given number
    pub fn render(&self, number: GenericNumber) -> SimpleResult<String> {
        self.to_formatter().render(number)
    }
}
