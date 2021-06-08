//! A library for reading well-defined datatypes from, ultimately, a [`Vec<u8>`].
//!
//! # Overview
//!
//! `h2datatype` is based on the [`H2Type`] type. An [`H2Type`] represents a
//! single contiguous chunk of memory with an optional alignment directive.
//!
//! An [`H2Type`] can be a simple type or a composite type. While these names
//! are somewhat arbitrary, the essential difference is that simple types are
//! fundamental building blocks, and composite types are made up of simple types
//! (and other composite types).
//!
//! An [`H2Type`] is somewhat abstract: it defines what the type is, how to
//! calculate its size, how to convert it to a string, and so on. To calculate
//! any of those, an [`Offset`] is required. An [`Offset`] can either be
//! abstract (a numeric offset value) or concrete (a buffer of bytes in the form
//! of a [`crate::generic_number::Context`]). Some types require a concrete
//! buffer to do anything useful (for example, while the length of an IPv4 value
//! doesn't change, the length of a UTF-8 character is based on the data).
//!
//! Pretty much all operations on an [`H2Type`] require an [`Offset`], but
//! whether can work with a [`Offset::Static`] or [`Offset::Dynamic`] depends on
//! the implementation.
//!
//! ## Resolving
//!
//! An [`H2Type`] can also be *resolved*. It's resolved against a particular
//! [`Offset`], and produces a [`ResolvedType`]. A [`ResolvedType`] has all the
//! same fields as a [`H2Type`], more or less, but they are now set in stone.
//! They can be fetched instantly, and have no chance of returning an error or
//! changing - the field has been resolved.
//!
//! ## Simple types
//!
//! A simple type, as mentioned above, is defined as a type that's not made up
//! of other types. The distinction isn't really all that meaningful, it's
//! simply a logical grouping.
//!
//! See the various classes in [`crate::datatype::simple`] for examples!
//!
//! ## Composite types
//!
//! A composite type is made up of other types. For example, a
//! [`composite::H2Array`] is a series of the same type, a
//! [`composite::H2Struct`] is a series of different types (with names), and a
//! [`composite::H2Union`] is a choice of overlapping values. These can be fully
//! recursive - an array can contain a struct which can contain an array and so
//! on, for as long as you like.
//!
//! ### String types
//!
//! A string type, which are defined in [`composite::string`], are a special
//! composite type. They're really just arrays of a value that can consume a
//! character type in some way to become a String.
//!
//! ## Alignment
//!
//! All [`H2Type`] values can be aligned. In the standard case, which is
//! [`Alignment::Loose`], an aligned value will always have a size that's a
//! multiple of the alignment value. That means that, for example, a string
//! that's 4-byte aligned will always take a total of 4, 8, 12, 16, ... bytes of
//! memory. If it ends off a byte boundary, the extra memory is consumed as part
//! of range but ultimately ignored.
//!
//! An alternative type of alignment is [`Alignment::Strict`], which is similar
//! to [`Alignment::Loose`], except that the start and end of the aligned value
//! must both be on an alignment boundary (relative to the start of the buffer).
//! That means if the alignment value is 4, all types must start on 0, 4, 8, ...
//! and will be padded to end on 4, 8, 12, ...
//!
//! # Examples
//!
//! ## Reading a 16-bit decimal value, signed
//!
//! ```
//! use libh2gb::datatype::*;
//! use libh2gb::datatype::simple::*;
//! use libh2gb::generic_number::*;
//!
//! // This is our buffer
//! let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
//!
//! // Create a dynamic offset (dynamic means it's linked to the actual data)
//! let offset = Offset::Dynamic(Context::new(&data));
//!
//! // Create the abstract type - this is an H2Type
//! let t = H2Number::new(GenericReader::I16(Endian::Big), DefaultFormatter::new());
//!
//! // It takes up two bytes of memory, including aligned (it's not aligned)
//! assert_eq!(2, t.actual_size(offset).unwrap());
//! assert_eq!(2, t.aligned_size(offset).unwrap());
//!
//! // Read the values at 0, 2, 4, and 8 bytes into the buffer
//! assert_eq!("0",      t.to_display(offset.at(0)).unwrap());
//! assert_eq!("32767",  t.to_display(offset.at(2)).unwrap());
//! assert_eq!("-32768", t.to_display(offset.at(4)).unwrap());
//! assert_eq!("-1",     t.to_display(offset.at(6)).unwrap());
//! ```
//!
//! ## Alignment
//!
//! ```
//! use libh2gb::datatype::*;
//! use libh2gb::datatype::simple::*;
//! use libh2gb::generic_number::*;
//!
//! // This is our buffer - the PP represents padding for alignment
//! let data = b"\x00\x00PP\x7f\xffPP\x80\x00PP\xff\xffPP".to_vec();
//!
//! // Create a dynamic offset (dynamic means it's linked to the actual data)
//! let offset = Offset::Dynamic(Context::new(&data));
//!
//! // Create the abstract type - this is an H2Type
//! let t = H2Number::new_aligned(
//!   Alignment::Loose(4), GenericReader::U16(Endian::Big),
//!   HexFormatter::pretty(),
//! );
//!
//! // It takes up two bytes of memory normally...
//! assert_eq!(2, t.actual_size(offset).unwrap());
//!
//! // ...but 4 bytes when aligned
//! assert_eq!(4, t.aligned_size(offset).unwrap());
//!
//! // Even though it takes up the extra space, the values don't change
//! assert_eq!("0x0000", t.to_display(offset.at(0)).unwrap());
//! assert_eq!("0x7fff", t.to_display(offset.at(4)).unwrap());
//! assert_eq!("0x8000", t.to_display(offset.at(8)).unwrap());
//! assert_eq!("0xffff", t.to_display(offset.at(12)).unwrap());
//! ```
//!
//! ## Composite types
//!
//! ```
//! use libh2gb::datatype::*;
//! use libh2gb::datatype::simple::*;
//! use libh2gb::datatype::composite::*;
//! use libh2gb::generic_number::*;
//!
//! // This is our buffer - the PP represents padding for alignment
//! let data = b"\x00\x00PP\x7f\xffPP\x80\x00PP\xff\xffPP".to_vec();
//!
//! // Create a dynamic offset (dynamic means it's linked to the actual data)
//! let offset = Offset::Dynamic(Context::new(&data));
//!
//! // Create an array of 4 elements, each of which is padded to 4 bytes
//! let t = H2Array::new(4, H2Number::new_aligned(
//!   Alignment::Loose(4), GenericReader::U16(Endian::Big),
//!   HexFormatter::pretty(),
//! )).unwrap();
//!
//! // The array takes up 16 bytes of memory, aligned and not
//! assert_eq!(16, t.actual_size(offset).unwrap());
//! assert_eq!(16, t.aligned_size(offset).unwrap());
//!
//! // Even though it takes up the extra space, the values don't change
//! assert_eq!("[ 0x0000, 0x7fff, 0x8000, 0xffff ]", t.to_display(offset.at(0)).unwrap());
//! ```
//!
//! ## Dynamic array
//!
//! Unlike in most programming languages, an array can be made up of different-
//! sized elements, like length-prefixed strings.
//!
//! ```
//! use libh2gb::datatype::*;
//! use libh2gb::datatype::simple::*;
//! use libh2gb::datatype::simple::character::*;
//! use libh2gb::datatype::composite::*;
//! use libh2gb::datatype::composite::string::*;
//! use libh2gb::generic_number::*;
//!
//! // This is our buffer - three strings with a one-byte length prefix
//! let data = b"\x02hi\x03bye\x04test".to_vec();
//!
//! // Create a dynamic offset (dynamic means it's linked to the actual data)
//! let offset = Offset::Dynamic(Context::new(&data));
//!
//! // Create an array of 3 elements, each of which is an LPString with a one-
//! // byte length
//! let t = H2Array::new(3, LPString::new(
//!   // The length field is an 8-bit unsigned integer
//!   H2Number::new(GenericReader::U8, HexFormatter::pretty()),
//!
//!   // The character type is just simple ascii
//!   ASCII::new(StrictASCII::Strict),
//! ).unwrap()).unwrap();
//!
//! // The array takes up 12 bytes of memory, all-in
//! assert_eq!(12, t.actual_size(offset).unwrap());
//!
//! // Even though it takes up the extra space, the values don't change
//! assert_eq!("[ \"hi\", \"bye\", \"test\" ]", t.to_display(offset).unwrap());
//! ```
//! # Things left to do
//!
//! Things I want to add:
//!
//! * Parse out structs and such from a C file
//! * Make sure I'm not converting u64 to usize
//! * Do I need H2Pointer at all?
//! * If an array is static, simplify the size
//! * Maybe split out characters again?
//!
//! Types needed:
//! * Bitmap / flag
//! * DNS name
//! * Type/length/value
//! * x86 / x64 / other assembly languages

mod alignment;
pub use alignment::Alignment;

mod resolved_type;
pub use resolved_type::ResolvedType;

mod offset;
pub use offset::Offset;

mod h2typetrait;
pub use h2typetrait::H2TypeTrait;

mod h2type;
pub use h2type::{H2Types, H2Type};

pub mod simple;
pub use simple::*;

pub mod composite;
pub use composite::*;
