# libh2gb

A simple library for reading differently-sized integers and floats.

While writing h2gb, I needed a way to dynamically read integers from a
Vec of u8 bytes. Libraries like [`byteorder`] and [`io::Cursor`] nearly
have the right functionality, but weren't quite flexible enough.

This library defines a [`Context`] type, which is just a thin wrapper for a
[`std::io::Cursor`]. It's also a super cheap type that can be cloned as
needed.

To create an instance, you need to first define how the number will be
defined. Using the [`SizedDefinition`] enum, choose a size and a
[`Endian`]:

```rust
use libh2gb::sized_number::{SizedDefinition, Endian};

let d = SizedDefinition::U32(Endian::Big);
```

Once you have an instance of [`SizedDefinition`], it can convert a
[`Context`] into a string in a variety of formats - use `SizedDisplay` to
configure how it should convert:

```rust
use libh2gb::sized_number::*;

let buffer = b"ABCD".to_vec();
let context = Context::new_at(&buffer, 0);
let d = SizedDefinition::U32(Endian::Big);

assert_eq!("0x41424344", d.to_string(context, SizedDisplay::Hex(HexOptions::default())).unwrap());
assert_eq!("1094861636", d.to_string(context, SizedDisplay::Decimal).unwrap());
assert_eq!("0o10120441504", d.to_string(context, SizedDisplay::Octal(Default::default())).unwrap());
assert_eq!("0b01000001010000100100001101000100", d.to_string(context, SizedDisplay::Binary(Default::default())).unwrap());
assert_eq!("1.094861636e9", d.to_string(context, SizedDisplay::Scientific(Default::default())).unwrap());
```

The string conversion is designed to be "stamp"-able - you can define the
format once, then apply it in multiple places or with multiple formats! No
context or data is stored as part of the type.

In addition to formatting a string, 64-bit and smaller unsigned instances
of [`SizedDefinition`] can be converted into [`u64`] (unsigned) integers, and
64-bit and smaller signed instances can be converted into [`i64`] (signed)
integers:

```rust
use libh2gb::sized_number::{Context, SizedDefinition, Endian, SizedDisplay, HexOptions, BinaryOptions, ScientificOptions};

let buffer = b"\x01\x02\x03\x04\x05\x06\x07\x08".to_vec();
let context = Context::new_at(&buffer, 0);
let d = SizedDefinition::U32(Endian::Big);

assert_eq!(0x01,               SizedDefinition::U8.to_u64(context).unwrap());
assert_eq!(0x0102,             SizedDefinition::U16(Endian::Big).to_u64(context).unwrap());
assert_eq!(0x01020304,         SizedDefinition::U32(Endian::Big).to_u64(context).unwrap());
assert_eq!(0x0102030405060708, SizedDefinition::U64(Endian::Big).to_u64(context).unwrap());
```

License: MIT
