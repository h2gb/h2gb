use serde::{Serialize, Deserialize};

use crate::sized_number::{SizedOptions, BinaryOptions, DecimalOptions, EnumType, HexOptions, OctalOptions, ScientificOptions};

/// Display options with their associated configurations.
///
/// This is the core for configuring the output. It tries to make the best
/// decisions based on the datatype. When displaying a padded hex value, for
/// example, it's padded to the exact width of the field, no matter what that
/// is.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SizedDisplay {
    /// Display in hexadecimal.
    ///
    /// Example:
    /// ```
    /// use libh2gb::sized_number::*;
    ///
    /// let buffer = b"\x00\xab".to_vec();
    /// let context = Context::new_at(&buffer, 0);
    /// let d = SizedDefinition::U16(Endian::Big);
    ///
    /// assert_eq!("0x00ab", d.to_string(context, SizedDisplay::Hex(HexOptions::default())).unwrap());
    ///
    /// assert_eq!("00AB", d.to_string(context, SizedDisplay::Hex(HexOptions {
    ///     uppercase: true,
    ///     prefix: false,
    ///     padded: true,
    /// })).unwrap());
    ///
    /// assert_eq!("0xab", d.to_string(context, SizedDisplay::Hex(HexOptions {
    ///     uppercase: false,
    ///     prefix: true,
    ///     padded: false,
    /// })).unwrap());
    ///
    /// ```
    Hex(HexOptions),

    /// Display in decimal. Whether the display is signed or not depends on the
    /// `SizedDefinition` type chosen.
    ///
    /// Example:
    /// ```
    /// use libh2gb::sized_number::*;
    ///
    /// let buffer = b"\xFF\xFF".to_vec();
    /// let context = Context::new_at(&buffer, 0);
    ///
    /// assert_eq!("255", SizedDefinition::U8.to_string(context, SizedDisplay::Decimal(Default::default())).unwrap());
    /// assert_eq!("-1", SizedDefinition::I8.to_string(context, SizedDisplay::Decimal(Default::default())).unwrap());
    ///
    /// ```
    Decimal(DecimalOptions),

    /// Display in octal.
    ///
    /// Example:
    /// ```
    /// use libh2gb::sized_number::*;
    ///
    /// let buffer = b"\x20".to_vec();
    /// let context = Context::new_at(&buffer, 0);
    ///
    /// assert_eq!("0o40", SizedDefinition::U8.to_string(context, SizedDisplay::Octal(Default::default())).unwrap());
    ///
    /// ```
    Octal(OctalOptions),

    /// Display in binary. Padding can be enabled with `BinaryOptions`
    ///
    /// Example:
    /// ```
    /// use libh2gb::sized_number::*;
    ///
    /// let buffer = b"\x01".to_vec();
    /// let context = Context::new_at(&buffer, 0);
    ///
    /// assert_eq!("0b00000001", SizedDefinition::U8.to_string(context, SizedDisplay::Binary(Default::default())).unwrap());
    /// ```
    Binary(BinaryOptions),

    /// Display in scientific / exponent notation. The case of `e` can be
    /// changed with `ScientificOptions`.
    ///
    /// Example:
    /// ```
    /// use libh2gb::sized_number::*;
    ///
    /// let buffer = b"\x64".to_vec();
    /// let context = Context::new_at(&buffer, 0);
    ///
    /// assert_eq!("1e2", SizedDefinition::U8.to_string(context, SizedDisplay::Scientific(Default::default())).unwrap());
    /// ```
    Scientific(ScientificOptions),

    /// Display as an 'enum' - a value selected from a list of common values.
    ///
    /// Example: XXX
    ///
    Enum(EnumType),
}

impl SizedDisplay {
    pub fn to_options(&self) -> Box<dyn SizedOptions> {
        match self {
            Self::Binary(o)     => Box::new(*o),
            Self::Decimal(o)    => Box::new(*o),
            Self::Enum(o)       => Box::new(*o),
            Self::Hex(o)        => Box::new(*o),
            Self::Octal(o)      => Box::new(*o),
            Self::Scientific(o) => Box::new(*o),
        }
    }
}
