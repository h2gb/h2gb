//! A library for reading and formatting differently-sized integers and floats.
//!
//! The bulk of functionality is split into three parts:
//!
//! * Datatypes - [`Integer`], [`Float`], and [`Character`], which represent
//!   datatypes and implement traits similar to the datatypes they represent
//!
//! * Readers - [`IntegerReader`], [`FloatReader`], and [`CharacterReader`],
//!   which make it easy to read any of the native types out of a [`Context`]
//!
//! * Renderers - [`IntegerRenderer`], [`FloatRenderer`], and
//!   [`CharacterRenderer`], which define how something is rendered. They are
//!   not instantiated directly, but through the variety of
//!   [formatters](/generic-number/src/generic_formatter/).

//! # Usage
//!
//! To use, you typically want to:
//!
//! * Create a [`Context`]
//! * Read a datatype (Integer / Float / Character) using one of the readers
//! * Render it using one of the renderers
//!
//! Here's an example:
//!
//! ```
//! use generic_number::*;
//!
//! // Create a buffer
//! let buffer = b"\x01\x23\x45\x67".to_vec();
//!
//! // Create a context that points to the start of the buffer
//! let context = Context::new_at(&buffer, 0);
//!
//! // Create a reader that knows how to read a U32 big endian value - this
//! // reader can be serialized and used later!
//! let reader = IntegerReader::U32(Endian::Big);
//!
//! // Read from the context into a generic number - this number can be
//! // serialized and used later!
//! let number = reader.read(context).unwrap();
//!
//! // Display it using different formatters (these use the pretty defaults) -
//! // these formatters can also be serialized!
//! assert_eq!("0x01234567",                         HexFormatter::pretty_integer().render(number));
//! assert_eq!("19088743",                           DefaultFormatter::new_integer().render(number));
//! assert_eq!("0o110642547",                        OctalFormatter::pretty_integer().render(number));
//! assert_eq!("0b00000001001000110100010101100111", BinaryFormatter::pretty_integer().render(number));
//! assert_eq!("1.9088743e7",                        ScientificFormatter::pretty_integer().render(number));
//! ```

mod helpers;
pub use helpers::*;

mod generic_formatter;
pub use generic_formatter::*;

mod integer;
pub use integer::*;

mod integer_reader;
pub use integer_reader::*;

mod integer_renderer;
pub use integer_renderer::*;

mod float;
pub use float::*;

mod float_reader;
pub use float_reader::*;

mod float_renderer;
pub use float_renderer::*;

mod character;
pub use character::*;

mod character_reader;
pub use character_reader::*;

mod character_renderer;
pub use character_renderer::*;
