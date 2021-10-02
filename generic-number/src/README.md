***Note: This file was automatically generated from lib.rs or mod.rs***

A library for reading and formatting differently-sized integers and floats.

The bulk of functionality is split into a couple parts:
XXX This is old documentation
* [`GenericReader`] - Reads any primitive type from a [`Context`]
* [`GenericNumber`] - Represents any primitive type
* [`GenericFormatter`] - Renders a [`GenericNumber`] with user-configurable options

Both [`GenericNumber`] and [`GenericFormatter`] are serializable, which is
what makes it really useful for h2gb!

## Usage

To use, you typically read a value from a buffer using a [`GenericReader`],
then display it using a [`GenericFormatter`]:

```rust
use generic_number::*;

// Create a buffer
let buffer = b"\x01\x23\x45\x67".to_vec();

// Create a context that points to the start of the buffer
let context = Context::new_at(&buffer, 0);

// Create a reader that knows how to read a U32 big endian value - this
// reader can be serialized and used later!
let reader = IntegerReader::U32(Endian::Big);

// Read from the context into a generic number - this number can be
// serialized and used later!
let number = reader.read(context).unwrap();

// Display it using different formatters (these use the pretty defaults) -
// these formatters can also be serialized!
assert_eq!("0x01234567",                         HexFormatter::pretty_integer().render(number));
assert_eq!("19088743",                           DefaultFormatter::new_integer().render(number));
assert_eq!("0o110642547",                        OctalFormatter::pretty_integer().render(number));
assert_eq!("0b00000001001000110100010101100111", BinaryFormatter::pretty_integer().render(number));
assert_eq!("1.9088743e7",                        ScientificFormatter::pretty_integer().render(number));
```

License: MIT
