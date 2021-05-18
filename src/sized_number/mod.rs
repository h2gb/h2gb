//! A simple library for reading differently-sized integers and floats.
//!
//! While writing h2gb, I needed a way to dynamically read integers from a
//! Vec of u8 bytes. Libraries like [`byteorder`] and [`io::Cursor`] nearly
//! have the right functionality, but weren't quite flexible enough.
//!
//! This library defines a [`Context`] type, which is just a thin wrapper for a
//! [`std::io::Cursor`]. It's also a super cheap type that can be cloned as
//! needed.
//!
//! To create an instance, you need to first define how the number will be
//! defined. Using the [`SizedDefinition`] enum, choose a size and a
//! [`Endian`]:
//!
//! ```
//! use libh2gb::sized_number::{SizedDefinition, Endian};
//!
//! let d = SizedDefinition::U32(Endian::Big);
//! ```
//!
//! Once you have an instance of [`SizedDefinition`], it can convert a
//! [`Context`] into a string in a variety of formats - use `SizedDisplay` to
//! configure how it should convert:
//!
//! ```
//! use libh2gb::sized_number::*;
//!
//! let buffer = b"ABCD".to_vec();
//! let context = Context::new_at(&buffer, 0);
//! let d = SizedDefinition::U32(Endian::Big);
//!
//! assert_eq!("0x41424344", d.to_string(context, SizedDisplay::Hex(HexOptions::default())).unwrap());
//! assert_eq!("1094861636", d.to_string(context, SizedDisplay::Decimal).unwrap());
//! assert_eq!("0o10120441504", d.to_string(context, SizedDisplay::Octal(Default::default())).unwrap());
//! assert_eq!("0b01000001010000100100001101000100", d.to_string(context, SizedDisplay::Binary(Default::default())).unwrap());
//! assert_eq!("1.094861636e9", d.to_string(context, SizedDisplay::Scientific(Default::default())).unwrap());
//! ```
//!
//! The string conversion is designed to be "stamp"-able - you can define the
//! format once, then apply it in multiple places or with multiple formats! No
//! context or data is stored as part of the type.
//!
//! In addition to formatting a string, 64-bit and smaller unsigned instances
//! of [`SizedDefinition`] can be converted into [`u64`] (unsigned) integers, and
//! 64-bit and smaller signed instances can be converted into [`i64`] (signed)
//! integers:
//!
//! ```
//! use libh2gb::sized_number::{Context, SizedDefinition, Endian, SizedDisplay, HexOptions, BinaryOptions, ScientificOptions};
//!
//! let buffer = b"\x01\x02\x03\x04\x05\x06\x07\x08".to_vec();
//! let context = Context::new_at(&buffer, 0);
//! let d = SizedDefinition::U32(Endian::Big);
//!
//! assert_eq!(0x01,               SizedDefinition::U8.to_u64(context).unwrap());
//! assert_eq!(0x0102,             SizedDefinition::U16(Endian::Big).to_u64(context).unwrap());
//! assert_eq!(0x01020304,         SizedDefinition::U32(Endian::Big).to_u64(context).unwrap());
//! assert_eq!(0x0102030405060708, SizedDefinition::U64(Endian::Big).to_u64(context).unwrap());
//! ```

use simple_error::{SimpleResult, bail};
use std::fmt::{LowerHex, LowerExp, Octal, Display};
use std::mem;

use serde::{Serialize, Deserialize};

mod context;
pub use context::{Context, Endian};

pub mod binary_options;
pub use binary_options::*;

pub mod enum_options;
pub use enum_options::*;

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

/// Configure display options for [`SizedDisplay::Hex`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HexOptions {
    /// Print hex characters uppercase - `1A2B` vs `1a2b`.
    pub uppercase: bool,

    /// Prefix hex strings with `0x`
    pub prefix: bool,

    /// Zero-pad hex strings to the full width - `0001` vs `1`)
    pub padded: bool,
}

impl Default for HexOptions {
    fn default() -> Self {
        Self {
            uppercase: false,
            prefix: true,
            padded: true,
        }
    }
}

/// Configure display options for [`SizedDisplay::Octal`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OctalOptions {
    /// Prefix octal strings with `0o`
    pub prefix: bool,

    /// Zero-pad octal strings to the full width - `0001` vs `1`)
    pub padded: bool,
}

impl Default for OctalOptions {
    fn default() -> Self {
        Self {
            prefix: true,
            padded: false,
        }
    }
}

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
    /// assert_eq!("255", SizedDefinition::U8.to_string(context, SizedDisplay::Decimal).unwrap());
    /// assert_eq!("-1", SizedDefinition::I8.to_string(context, SizedDisplay::Decimal).unwrap());
    ///
    /// ```
    Decimal,

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

/// Define how data is read from a Context.
///
/// This is the core of `sized_number` - it's how the numbers are defined in
/// memory.
///
/// The options all pretty cleanly map to the equivalent datatypes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SizedDefinition {
    /// Unsigned 8-bit integer
    U8,

    /// Unsigned 16-bit integer
    U16(Endian),

    /// Unsigned 32-bit integer
    U32(Endian),

    /// Unsigned 64-bit integer
    U64(Endian),

    /// Unsigned 128-bit integer
    U128(Endian),

    /// Signed 8-bit integer
    I8,

    /// Signed 16-bit integer
    I16(Endian),

    /// Signed 32-bit integer
    I32(Endian),

    /// Signed 64-bit integer
    I64(Endian),

    /// Signed 128-bit integer
    I128(Endian),

    /// Signed 32-bit (aka, single precision) floating point.
    ///
    /// Note: floats can only be displayed as `SizedDisplay::Decimal` or
    /// `SizedDisplay::Scientific`.
    F32(Endian),

    /// Signed 64-bit (aka, double precision) floating point
    ///
    /// Note: floats can only be displayed as `SizedDisplay::Decimal` or
    /// `SizedDisplay::Scientific`.
    F64(Endian),
}

/// An internal function to help with displaying hex.
///
/// Unfortunately, I don't know of a way to require both [`UpperHex`] and
/// [`LowerHex`] traits, so I do some manual formatting :-/
fn display_hex(v: Box<dyn LowerHex>, options: HexOptions) -> String {
    let v = v.as_ref();

    let mut h = match options.padded {
        // No padding is easy
        false => format!("{:x}",   v),

        // Padding requires a bit more tinkering to do dynamically
        true => {
            match (options.padded, mem::size_of_val(v) * 2) {
                (true, 2)   => format!(  "{:02x}",  v),
                (true, 4)   => format!(  "{:04x}",  v),
                (true, 8)   => format!(  "{:08x}",  v),
                (true, 16)  => format!(  "{:016x}", v),
                (true, 32)  => format!(  "{:032x}", v),

                // When not padded, or in doubt about length, just print normally
                (_, _)      => format!(  "{:x}",     v),
            }
        }
    };

    // There's no way to make the parameter both LowerHex and UpperHex
    if options.uppercase {
        h = h.to_uppercase();
    }

    if options.prefix {
        h = format!("0x{}", h);
    }

    h
}

/// An internal function to help with displaying decimal
fn display_decimal(v: Box<dyn Display>) -> String {
    format!("{}", v.as_ref())
}

/// An internal function to help with displaying octal
fn display_octal(v: Box<dyn Octal>, options: OctalOptions) -> String {
    let v = v.as_ref();

    if options.padded {
        match (options.prefix, mem::size_of_val(v)) {
            (false, 1)  => format!("{:03o}", v),
            (false, 2)  => format!("{:06o}", v),
            (false, 4)  => format!("{:011o}", v),
            (false, 8)  => format!("{:022o}", v),
            (false, 16) => format!("{:043o}", v),
            (false, _) => format!("{:o}", v),

            (true,  1)  => format!("0o{:03o}", v),
            (true,  2)  => format!("0o{:06o}", v),
            (true,  4)  => format!("0o{:011o}", v),
            (true,  8)  => format!("0o{:022o}", v),
            (true,  16) => format!("0o{:043o}", v),

            (true,   _) => format!("0o{:o}", v),
        }
    } else {
        match options.prefix {
            false => format!("{:o}", v),
            true  => format!("0o{:o}", v),
        }
    }
}

/// An internal function to help with displaying scientific / exponential
/// notation.
fn display_scientific(v: Box<dyn LowerExp>, options: ScientificOptions) -> String {
    let mut v = format!("{:e}", v.as_ref());

    if options.uppercase {
        v = v.to_uppercase();
    }

    v
}

impl SizedDefinition {
    /// Returns the size, in bytes, of the current type.
    pub fn size(self) -> u64 {
        match self {
            Self::U8      => 1,
            Self::U16(_)  => 2,
            Self::U32(_)  => 4,
            Self::U64(_)  => 8,
            Self::U128(_) => 16,

            Self::I8      => 1,
            Self::I16(_)  => 2,
            Self::I32(_)  => 4,
            Self::I64(_)  => 8,
            Self::I128(_) => 16,

            Self::F32(_)  => 4,
            Self::F64(_)  => 8,
        }
    }

    /// Read data from the context, based on the [`SizedDefinition`], and
    /// display it based on the `SizedDisplay`
    pub fn to_string(self, context: Context, display: SizedDisplay) -> SimpleResult<String> {
        match self {
            Self::U8 => {
                let v = Box::new(context.read_u8()?);
                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(options)       => Ok(options.u64_to_s(*v as u64)),
                }
            },

            Self::U16(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_u16(Endian::Big)?),
                    Endian::Little => Box::new(context.read_u16(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(options)       => Ok(options.u64_to_s(*v as u64)),
                }
            },

            Self::U32(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_u32(Endian::Big)?),
                    Endian::Little => Box::new(context.read_u32(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(options)       => Ok(options.u64_to_s(*v as u64)),
                }
            },

            Self::U64(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_u64(Endian::Big)?),
                    Endian::Little => Box::new(context.read_u64(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(options)       => Ok(options.u64_to_s(*v as u64)),
                }
            },

            Self::U128(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_u128(Endian::Big)?),
                    Endian::Little => Box::new(context.read_u128(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(_)             => bail!("128-bit values cannot be an enum element"),
                }
            },

            Self::I8 => {
                let v = Box::new(context.read_i8()?);

                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(options)       => Ok(options.i64_to_s(*v as i64)),
                }
            },

            Self::I16(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_i16(Endian::Big)?),
                    Endian::Little => Box::new(context.read_i16(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(options)       => Ok(options.i64_to_s(*v as i64)),
                }
            },

            Self::I32(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_i32(Endian::Big)?),
                    Endian::Little => Box::new(context.read_i32(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(options)       => Ok(options.i64_to_s(*v as i64)),
                }
            },

            Self::I64(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_i64(Endian::Big)?),
                    Endian::Little => Box::new(context.read_i64(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(options)       => Ok(options.i64_to_s(*v as i64)),
                }
            },

            Self::I128(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_i128(Endian::Big)?),
                    Endian::Little => Box::new(context.read_i128(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(options)        => Ok(display_hex(v, options)),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(options)      => Ok(display_octal(v, options)),
                    SizedDisplay::Binary(options)     => Ok(options.to_s(v)),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(_)             => bail!("128-bit values cannot be an enum element"),
                }
            },

            Self::F32(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_f32(Endian::Big)?),
                    Endian::Little => Box::new(context.read_f32(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(_)              => bail!("Floats can't be displayed as hex"),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(_)            => bail!("Floats can't be displayed as octal"),
                    SizedDisplay::Binary(_)           => bail!("Floats can't be displayed as binary"),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(_)             => bail!("Floats cannot be an enum element"),
                }
            },

            Self::F64(endian) => {
                let v = match endian {
                    Endian::Big => Box::new(context.read_f64(Endian::Big)?),
                    Endian::Little => Box::new(context.read_f64(Endian::Little)?),
                };

                match display {
                    SizedDisplay::Hex(_)              => bail!("Floats can't be displayed as hex"),
                    SizedDisplay::Decimal             => Ok(display_decimal(v)),
                    SizedDisplay::Octal(_)            => bail!("Floats can't be displayed as octal"),
                    SizedDisplay::Binary(_)           => bail!("Floats can't be displayed as binary"),
                    SizedDisplay::Scientific(options) => Ok(display_scientific(v, options)),
                    SizedDisplay::Enum(_)             => bail!("Floats cannot be an enum element"),
                }
            },
        }
    }

    /// Returns `true` for variants that can succeed in [`to_u64`]
    pub fn can_be_u64(self) -> bool {
        match self {
            Self::U8      => true,
            Self::U16(_)  => true,
            Self::U32(_)  => true,
            Self::U64(_)  => true,
            Self::U128(_) => false,

            Self::I8      => false,
            Self::I16(_)  => false,
            Self::I32(_)  => false,
            Self::I64(_)  => false,
            Self::I128(_) => false,

            Self::F32(_)  => false,
            Self::F64(_)  => false,
        }
    }

    /// Returns `true` for variants that can succeed in [`to_i64`]
    pub fn can_be_i64(self) -> bool {
        match self {
            Self::U8      => false,
            Self::U16(_)  => false,
            Self::U32(_)  => false,
            Self::U64(_)  => false,
            Self::U128(_) => false,

            Self::I8      => true,
            Self::I16(_)  => true,
            Self::I32(_)  => true,
            Self::I64(_)  => true,
            Self::I128(_) => false,

            Self::F32(_)  => false,
            Self::F64(_)  => false,
        }
    }

    /// Returns `true` for variants that can be displayed as an Enum
    pub fn can_be_enum(self) -> bool {
        self.can_be_u64() || self.can_be_i64()
    }

    /// Convert to an unsigned 64-bit value, if possible.
    ///
    /// Only unsigned values of 64-bits or less can be converted to a [`u64`].
    /// Everything else will return an error - we don't typecast signed to
    /// unsigned.
    pub fn to_u64(self, context: Context) -> SimpleResult<u64> {
        match self {
            Self::U8          => Ok(context.read_u8()? as u64),
            Self::U16(endian) => Ok(context.read_u16(endian)? as u64),
            Self::U32(endian) => Ok(context.read_u32(endian)? as u64),
            Self::U64(endian) => Ok(context.read_u64(endian)? as u64),

            // None of these can become u32
            Self::U128(_) => bail!("Can't convert u128 into u64"),

            Self::I8      => bail!("Can't convert i8 (signed) into u64"),
            Self::I16(_)  => bail!("Can't convert i16 (signed) into u64"),
            Self::I32(_)  => bail!("Can't convert i32 (signed) into u64"),
            Self::I64(_)  => bail!("Can't convert i64 (signed) into u64"),
            Self::I128(_) => bail!("Can't convert i128 (signed) into u64"),

            Self::F32(_)  => bail!("Can't convert floating point into u64"),
            Self::F64(_)  => bail!("Can't convert floating point into u64"),
        }
    }

    /// Convert to a signed 64-bit value, if possible.
    ///
    /// This will correctly extend the sign. So, for example, reading a
    /// `SizedDefinition::I8` with a value of `FF` will convert to the [`i64`]
    /// value `-1`, or `0xFFFFFFFFFFFFFFFF`.
    ///
    /// Only signed values of 64-bits or less can be converted to an [`i64`].
    /// Everything else will return an error - we don't typecast unsigned to
    /// signed.
    pub fn to_i64(self, context: Context) -> SimpleResult<i64> {
        match self {
            // Don't let unsigned values become signed
            Self::U8      => bail!("Can't convert i8 (signed) into i64"),
            Self::U16(_)  => bail!("Can't convert i16 (signed) into i64"),
            Self::U32(_)  => bail!("Can't convert i32 (signed) into i64"),
            Self::U64(_)  => bail!("Can't convert i64 (signed) into i64"),
            Self::U128(_) => bail!("Can't convert i128 (signed) into i64"),

            Self::I8 =>          Ok(context.read_i8()? as i64),
            Self::I16(endian) => Ok(context.read_i16(endian)? as i64),
            Self::I32(endian) => Ok(context.read_i32(endian)? as i64),
            Self::I64(endian) => Ok(context.read_i64(endian)? as i64),

            // 128 bit can't go into 64 bit
            Self::I128(_) => bail!("Can't convert u128 into i64"),

            // Float certainly can't
            Self::F32(_)  => bail!("Can't convert floating point into i64"),
            Self::F64(_)  => bail!("Can't convert floating point into i64"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    #[test]
    fn test_hex_u8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "0"),
            (   0,    true,       false,   false,    "0"),
            (   0,    false,      true,    false,    "0x0"),
            (   0,    false,      false,   true,     "00"),
            (   0,    true,       true,    true,     "0x00"),

            // index  uppercase   prefix   padded    expected
            (   1,    false,      false,   false,    "7f"),
            (   1,    true,       false,   false,    "7F"),
            (   1,    false,      true,    false,    "0x7f"),
            (   1,    false,      false,   true,     "7f"),
            (   1,    true,       true,    true,     "0x7F"),

            // index  uppercase   prefix   padded    expected
            (   2,    false,      false,   false,    "80"),
            (   2,    true,       false,   false,    "80"),
            (   2,    false,      true,    false,    "0x80"),
            (   2,    false,      false,   true,     "80"),
            (   2,    true,       true,    true,     "0x80"),

            // index  uppercase   prefix   padded    expected
            (   3,    false,      false,   false,    "ff"),
            (   3,    true,       false,   false,    "FF"),
            (   3,    false,      true,    false,    "0xff"),
            (   3,    false,      false,   true,     "ff"),
            (   3,    true,       true,    true,     "0xFF"),

        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U8.to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u16() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "0"),
            (   0,    true,       false,   false,    "0"),
            (   0,    false,      true,    false,    "0x0"),
            (   0,    false,      false,   true,     "0000"),
            (   0,    true,       true,    true,     "0x0000"),

            // index  uppercase   prefix   padded    expected
            (   2,    false,      false,   false,    "1234"),
            (   2,    true,       false,   false,    "1234"),
            (   2,    false,      true,    false,    "0x1234"),
            (   2,    false,      false,   true,     "1234"),
            (   2,    true,       true,    true,     "0x1234"),

            // index  uppercase   prefix   padded    expected
            (   4,    false,      false,   false,    "ffff"),
            (   4,    true,       false,   false,    "FFFF"),
            (   4,    false,      true,    false,    "0xffff"),
            (   4,    false,      false,   true,     "ffff"),
            (   4,    true,       true,    true,     "0xFFFF"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U16(Endian::Big).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u32() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "1234"),
            (   0,    true,       false,   false,    "1234"),
            (   0,    false,      true,    false,    "0x1234"),
            (   0,    false,      false,   true,     "00001234"),
            (   0,    true,       true,    true,     "0x00001234"),

            // index  uppercase   prefix   padded    expected
            (   4,    false,      false,   false,    "ffffffff"),
            (   4,    true,       false,   false,    "FFFFFFFF"),
            (   4,    false,      true,    false,    "0xffffffff"),
            (   4,    false,      false,   true,     "ffffffff"),
            (   4,    true,       true,    true,     "0xFFFFFFFF"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u64_big_endian() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "1234ffffffff"),
            (   0,    true,       false,   false,    "1234FFFFFFFF"),
            (   0,    false,      true,    false,    "0x1234ffffffff"),
            (   0,    false,      false,   true,     "00001234ffffffff"),
            (   0,    true,       true,    true,     "0x00001234FFFFFFFF"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u64_little_endian() -> SimpleResult<()> {
        let data = b"\x00\x12\x34\xFF\xFF\xFF\xFF\x00".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "ffffffff341200"),
            (   0,    true,       false,   false,    "FFFFFFFF341200"),
            (   0,    false,      true,    false,    "0xffffffff341200"),
            (   0,    false,      false,   true,     "00ffffffff341200"),
            (   0,    true,       true,    true,     "0x00FFFFFFFF341200"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U64(Endian::Little).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_hex_u128_big_endian() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00\x11\x22\x33\x44\x55\x66\x77\x88\x99\xaa\xbb\xcc\xdd\xee\xff".to_vec();

        let tests = vec![
            // index  uppercase   prefix   padded    expected
            (   0,    false,      false,   false,    "1"),
            (   0,    true,       false,   false,    "1"),
            (   0,    false,      true,    false,    "0x1"),
            (   0,    false,      false,   true,     "00000000000000000000000000000001"),
            (   0,    true,       true,    true,     "0x00000000000000000000000000000001"),

            // index  uppercase   prefix   padded    expected
            (   16,    false,      false,   false,    "112233445566778899aabbccddeeff"),
            (   16,    true,       false,   false,    "112233445566778899AABBCCDDEEFF"),
            (   16,    false,      true,    false,    "0x112233445566778899aabbccddeeff"),
            (   16,    false,      false,   true,     "00112233445566778899aabbccddeeff"),
            (   16,    true,       true,    true,     "0x00112233445566778899AABBCCDDEEFF"),
        ];

        for (index, uppercase, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U128(Endian::Big).to_string(
                    context,
                    SizedDisplay::Hex(HexOptions {
                        uppercase: uppercase,
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_u8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   1,    "127"),
            (   2,    "128"),
            (   3,    "255"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U8.to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_i8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   1,    "127"),
            (   2,    "-128"),
            (   3,    "-1"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I8.to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_u16() -> SimpleResult<()> {
        let data = b"\x00\xFF\x00\x01\x00\x00\xFF\xFF".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "255"),
            (   2,    "1"),
            (   4,    "0"),
            (   6,    "65535"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U16(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_u32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\xff\xff\xff\xff\x7f\xff\xff\xff\x80\x00\x00\x00".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   4,    "4294967295"),
            (   8,    "2147483647"),
            (  12,    "2147483648"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_i32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\xff\xff\xff\xff\x7f\xff\xff\xff\x80\x00\x00\x00".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   4,    "-1"),
            (   8,    "2147483647"),
            (  12,    "-2147483648"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_i64() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x7f\xff\xff\xff\xff\xff\xff\xff\x80\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff".to_vec();

        let tests = vec![
            // index  expected
            (  0,    "0"),
            (  8,    "9223372036854775807"),
            (  16,   "-9223372036854775808"),
            (  24,   "-1"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_u128() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  expected
            (  0,    "0"),
            (  16,   "340282366920938463463374607431768211455"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U128(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_i128() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  expected
            (  0,    "0"),
            (  16,   "-1"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I128(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_octal_u8() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF".to_vec();

        let tests = vec![
            // index  prefix   padded   expected
            (   0,    false,   false,   "0"),
            (   1,    false,   false,   "177"),
            (   2,    false,   false,   "200"),
            (   3,    false,   false,   "377"),

            // index  prefix   padded   expected
            (   0,    false,   true,    "000"),
            (   1,    false,   true,    "177"),
            (   2,    false,   true,    "200"),
            (   3,    false,   true,    "377"),

            // index  prefix   padded   expected
            (   0,    true,    false,   "0o0"),
            (   1,    true,    false,   "0o177"),
            (   2,    true,    false,   "0o200"),
            (   3,    true,    false,   "0o377"),

            // index  prefix   padded   expected
            (   0,    true,    true,    "0o000"),
            (   1,    true,    true,    "0o177"),
            (   2,    true,    true,    "0o200"),
            (   3,    true,    true,    "0o377"),
        ];

        for (index, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U8.to_string(
                    context,
                    SizedDisplay::Octal(OctalOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_octal_u16() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  prefix   padded   expected
            (   0,    false,   false,   "0"),
            (   2,    false,   false,   "11064"),
            (   4,    false,   false,   "177777"),

            // index  prefix   padded   expected
            (   0,    false,   true,    "000000"),
            (   2,    false,   true,    "011064"),
            (   4,    false,   true,    "177777"),

            // index  prefix   padded   expected
            (   0,    true,    false,   "0o0"),
            (   2,    true,    false,   "0o11064"),
            (   4,    true,    false,   "0o177777"),

            // index  prefix   padded   expected
            (   0,    true,    true,    "0o000000"),
            (   2,    true,    true,    "0o011064"),
            (   4,    true,    true,    "0o177777"),
        ];

        for (index, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U16(Endian::Big).to_string(
                    context,
                    SizedDisplay::Octal(OctalOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_octal_u32() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  prefix  padded   expected
            (   0,    false,  false,   "11064"),
            (   2,    false,  false,   "2215177777"),
            (   4,    false,  false,   "37777777777"),

            // index  prefix  padded   expected
            (   0,    false,  true,   "00000011064"),
            (   2,    false,  true,   "02215177777"),
            (   4,    false,  true,   "37777777777"),

            // index  prefix  padded   expected
            (   0,    true,  false,   "0o11064"),
            (   2,    true,  false,   "0o2215177777"),
            (   4,    true,  false,   "0o37777777777"),

            // index  prefix  padded   expected
            (   0,    true,  true,     "0o00000011064"),
            (   2,    true,  true,     "0o02215177777"),
            (   4,    true,  true,     "0o37777777777"),
        ];

        for (index, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Octal(OctalOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_octal_u64() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\x34\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index  prefix  padded  expected
            (   0,    false,  false,  "443237777777777"),
            (   8,    false,  false,  "1777777777777777777777"),

            // index  prefix  padded  expected
            (   0,    false,  true,   "0000000443237777777777"),
            (   8,    false,  true,   "1777777777777777777777"),

            // index  prefix  padded  expected
            (   0,    true,   false,  "0o443237777777777"),
            (   8,    true,   false,  "0o1777777777777777777777"),

            // index  prefix  padded  expected
            (   0,    true,   true,   "0o0000000443237777777777"),
            (   8,    true,   true,   "0o1777777777777777777777"),

        ];

        for (index, prefix, padded, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Octal(OctalOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_binary_i8() -> SimpleResult<()> {
        let data = b"\x00\x00\x12\xab\xFF\xFF\xFF\xFF".to_vec();

        let tests = vec![
            // index    prefix      padded   expected
            (   0,      true,       true,    "0b00000000"),
            (   1,      true,       true,    "0b00000000"),
            (   2,      true,       true,    "0b00010010"),
            (   3,      true,       true,    "0b10101011"),
            (   4,      true,       true,    "0b11111111"),
            (   5,      true,       true,    "0b11111111"),

            (   0,      false,      false,   "0"),
            (   1,      false,      false,   "0"),
            (   2,      false,      false,   "10010"),
            (   3,      false,      false,   "10101011"),
            (   4,      false,      false,   "11111111"),
            (   5,      false,      false,   "11111111"),
        ];

        for (index, padded, prefix, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U8.to_string(
                    context,
                    SizedDisplay::Binary(BinaryOptions {
                        prefix: prefix,
                        padded: padded,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_scientific_u32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x7f\xff\xff\xff\x80\x00\x00\x00\xff\xff\xff\xff".to_vec();

        let tests = vec![
            // index  uppercase  expected
            (   0,    false,     "0e0"),
            (   4,    false,     "2.147483647e9"),
            (   8,    false,     "2.147483648e9"),
            (  12,    false,     "4.294967295e9"),
            (   0,    true,      "0E0"),
            (   4,    true,      "2.147483647E9"),
            (   8,    true,      "2.147483648E9"),
            (  12,    true,      "4.294967295E9"),
        ];

        for (index, uppercase, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::U32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Scientific(ScientificOptions {
                        uppercase: uppercase,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_scientific_i32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x7f\xff\xff\xff\x80\x00\x00\x00\xff\xff\xff\xff".to_vec();

        let tests = vec![
            // index  uppercase  expected
            (   0,    false,     "0e0"),
            (   4,    false,     "2.147483647e9"),
            (   8,    false,     "-2.147483648e9"),
            (  12,    false,     "-1e0"),
            (   0,    true,      "0E0"),
            (   4,    true,      "2.147483647E9"),
            (   8,    true,      "-2.147483648E9"),
            (  12,    true,      "-1E0"),
        ];

        for (index, uppercase, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::I32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Scientific(ScientificOptions {
                        uppercase: uppercase,
                    })
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_f32() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\xff\xff\xff\xff\x41\xc8\x00\x00\x40\x48\xf5\xc3".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "0"),
            (   4,    "NaN"),
            (   8,    "25"), // From https://en.wikipedia.org/wiki/Single-precision_floating-point_format#Converting_from_single-precision_binary_to_decimal
            (  12,    "3.14"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::F32(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_f64_big_endian() -> SimpleResult<()> {
        // I wrote and disassembled a simple C program to get these strings.. double is hard
        let data = b"\x40\x09\x1e\xb8\x51\xeb\x85\x1f\x40\x09\x33\x33\x33\x33\x33\x33".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "3.14"),
            (   8,    "3.15"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::F64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_decimal_f64_little_endian() -> SimpleResult<()> {
        // I wrote and disassembled a simple C program to get these strings.. double is hard
        let data = b"\x1F\x85\xEB\x51\xB8\x1E\x09\x40\x33\x33\x33\x33\x33\x33\x09\x40".to_vec();

        let tests = vec![
            // index  expected
            (   0,    "3.14"),
            (   8,    "3.15"),
        ];

        for (index, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::F64(Endian::Little).to_string(
                    context,
                    SizedDisplay::Decimal
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_exponent_f64() -> SimpleResult<()> {
        // I wrote and disassembled a simple C program to get these strings.. double is hard
        let data = b"\x40\x09\x1e\xb8\x51\xeb\x85\x1f\x40\x09\x33\x33\x33\x33\x33\x33".to_vec();

        let tests = vec![
            // index  uppercase expected
            (   0,    false,    "3.14e0"),
            (   8,    false,    "3.15e0"),
            (   0,    true,     "3.14E0"),
            (   8,    true,     "3.15E0"),
        ];

        for (index, uppercase, expected) in tests {
            let context = Context::new_at(&data, index);

            assert_eq!(
                expected,
                SizedDefinition::F64(Endian::Big).to_string(
                    context,
                    SizedDisplay::Scientific(ScientificOptions {
                        uppercase: uppercase,
                    }),
                )?
            );
        }

        Ok(())
    }

    #[test]
    fn test_buffer_too_short() -> SimpleResult<()> {
        let data = b"".to_vec();
        assert!(SizedDefinition::I8.to_string(Context::new(&data), SizedDisplay::Decimal).is_err());

        let data = b"A".to_vec();
        assert!(SizedDefinition::I16(Endian::Big).to_string(Context::new(&data), SizedDisplay::Decimal).is_err());

        let data = b"AAA".to_vec();
        assert!(SizedDefinition::I32(Endian::Big).to_string(Context::new(&data), SizedDisplay::Decimal).is_err());

        let data = b"AAAAAAA".to_vec();
        assert!(SizedDefinition::I64(Endian::Big).to_string(Context::new(&data), SizedDisplay::Decimal).is_err());

        let data = b"AAAAAAAAAAAAAAA".to_vec();
        assert!(SizedDefinition::I128(Endian::Big).to_string(Context::new(&data), SizedDisplay::Decimal).is_err());

        Ok(())
    }

    #[test]
    fn test_to_u64() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03".to_vec();

        assert_eq!(0u64,   SizedDefinition::U8.to_u64(Context::new_at(&data, 0))?);
        assert_eq!(127u64, SizedDefinition::U8.to_u64(Context::new_at(&data, 1))?);
        assert_eq!(128u64, SizedDefinition::U8.to_u64(Context::new_at(&data, 2))?);
        assert_eq!(255u64, SizedDefinition::U8.to_u64(Context::new_at(&data, 3))?);

        assert_eq!(127u64,               SizedDefinition::U16(Endian::Big).to_u64(Context::new_at(&data, 0))?);
        assert_eq!(8356095u64,           SizedDefinition::U32(Endian::Big).to_u64(Context::new_at(&data, 0))?);
        assert_eq!(35889154747335171u64, SizedDefinition::U64(Endian::Big).to_u64(Context::new_at(&data, 0))?);

        assert!(SizedDefinition::U128(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::I8.to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::I16(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::I32(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::I64(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::F32(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::F64(Endian::Big).to_u64(Context::new_at(&data, 0)).is_err());

        Ok(())
    }

    #[test]
    fn test_to_i64() -> SimpleResult<()> {
        let data = b"\x00\x7F\x80\xFF\x00\x01\x02\x03\x80\x00\x00\x00\x00\x00\x00\x00".to_vec();

        assert_eq!(0i64,                    SizedDefinition::I8.to_i64(Context::new_at(&data, 0))?);
        assert_eq!(127i64,                  SizedDefinition::I8.to_i64(Context::new_at(&data, 1))?);
        assert_eq!(-128i64,                 SizedDefinition::I8.to_i64(Context::new_at(&data, 2))?);
        assert_eq!(-1i64,                   SizedDefinition::I8.to_i64(Context::new_at(&data, 3))?);

        assert_eq!(127i64,                  SizedDefinition::I16(Endian::Big).to_i64(Context::new_at(&data, 0))?);
        assert_eq!(-32768i64,               SizedDefinition::I16(Endian::Big).to_i64(Context::new_at(&data, 8))?);

        assert_eq!(8356095i64,              SizedDefinition::I32(Endian::Big).to_i64(Context::new_at(&data, 0))?);
        assert_eq!(-2147483648i64,          SizedDefinition::I32(Endian::Big).to_i64(Context::new_at(&data, 8))?);

        assert_eq!(35889154747335171i64,    SizedDefinition::I64(Endian::Big).to_i64(Context::new_at(&data, 0))?);
        assert_eq!(-9223372036854775808i64, SizedDefinition::I64(Endian::Big).to_i64(Context::new_at(&data, 8))?);

        assert!(SizedDefinition::I128(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::U8.to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::U16(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::U32(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::U64(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::F32(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());
        assert!(SizedDefinition::F64(Endian::Big).to_i64(Context::new_at(&data, 0)).is_err());

        Ok(())
    }
}
