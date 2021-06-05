use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::generic_number::GenericNumber;

mod binary_formatter;
pub use binary_formatter::*;

mod decimal_formatter;
pub use decimal_formatter::*;

mod enum_formatter;
pub use enum_formatter::*;

mod hex_formatter;
pub use hex_formatter::*;

mod octal_formatter;
pub use octal_formatter::*;

mod scientific_formatter;
pub use scientific_formatter::*;

pub trait GenericFormatterImpl {
    fn render(&self, number: GenericNumber) -> SimpleResult<String>;
}

/// Display options with their associated configurations.
///
/// This is the core for configuring the output. It tries to make the best
/// decisions based on the datatype. When displaying a padded hex value, for
/// example, it's padded to the exact width of the field, no matter what that
/// is.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GenericFormatter {
    /// Display in hexadecimal.
    ///
    /// Example:
    /// ```
    /// use libh2gb::generic_number::*;
    ///
    /// let buffer = b"\x00\xab".to_vec();
    /// let d = GenericReader::U16(Endian::Big);
    /// let number = d.read(Context::new_at(&buffer, 0)).unwrap();
    ///
    /// assert_eq!("0x00ab", HexFormatter::pretty().render(number).unwrap());
    /// assert_eq!("00AB", HexFormatter::new(true,  false, true ).render(number).unwrap());
    /// assert_eq!("0xab", HexFormatter::new(false, true,  false).render(number).unwrap());
    ///
    /// ```
    Hex(HexFormatter),

    /// Display in decimal. Whether the display is signed or not depends on the
    /// `GenericReader` type chosen.
    ///
    /// Example:
    /// ```
    /// use libh2gb::generic_number::*;
    ///
    /// let buffer = b"\xFF\xFF".to_vec();
    /// let unsigned = GenericReader::U8.read(Context::new_at(&buffer, 0)).unwrap();
    /// assert_eq!("255", DecimalFormatter::new().render(unsigned).unwrap());
    ///
    /// let signed   = GenericReader::I8.read(Context::new_at(&buffer, 0)).unwrap();
    /// assert_eq!("-1", DecimalFormatter::new().render(signed).unwrap());
    ///
    /// ```
    Decimal(DecimalFormatter),

    /// Display in octal.
    ///
    /// Example:
    /// ```
    /// use libh2gb::generic_number::*;
    ///
    /// let buffer = b"\x20".to_vec();
    /// let context = Context::new_at(&buffer, 0);
    /// let number = GenericReader::U8.read(context).unwrap();
    ///
    /// assert_eq!("0o40", OctalFormatter::pretty().render(number).unwrap());
    ///
    /// ```
    Octal(OctalFormatter),

    /// Display in binary. Padding can be enabled with `BinaryFormatter`
    ///
    /// Example:
    /// ```
    /// use libh2gb::generic_number::*;
    ///
    /// let buffer = b"\x01".to_vec();
    /// let context = Context::new_at(&buffer, 0);
    /// let number = GenericReader::U8.read(context).unwrap();
    ///
    /// assert_eq!("0b00000001", BinaryFormatter::pretty().render(number).unwrap());
    /// ```
    Binary(BinaryFormatter),

    /// Display in scientific / exponent notation. The case of `e` can be
    /// changed with `ScientificFormatter`.
    ///
    /// Example:
    /// ```
    /// use libh2gb::generic_number::*;
    ///
    /// let buffer = b"\x64".to_vec();
    /// let context = Context::new_at(&buffer, 0);
    /// let number = GenericReader::U8.read(context).unwrap();
    ///
    /// assert_eq!("1e2", ScientificFormatter::pretty().render(number).unwrap());
    /// ```
    Scientific(ScientificFormatter),

    /// Display as an 'enum' - a value selected from a list of common values.
    ///
    /// Example: XXX
    ///
    Enum(EnumFormatter),
}

impl GenericFormatter {
    pub fn to_formatter(&self) -> Box<dyn GenericFormatterImpl> {
        match self {
            Self::Binary(o)     => Box::new(*o),
            Self::Decimal(o)    => Box::new(*o),
            Self::Enum(o)       => Box::new(*o),
            Self::Hex(o)        => Box::new(*o),
            Self::Octal(o)      => Box::new(*o),
            Self::Scientific(o) => Box::new(*o),
        }
    }

    pub fn render(&self, number: GenericNumber) -> SimpleResult<String> {
        self.to_formatter().render(number)
    }
}
