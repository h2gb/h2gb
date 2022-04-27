***Note: This file was automatically generated from [h2datatype/src/data/mod.rs](/h2datatype/src/data/mod.rs)***

Pre-canned datatypes for easier analysis.

This document will focus on usage from the perspective of somebody writing
a module. How data gets into this originally is outside the scope (check
out the documentation for [`DataEntry`] if you're interested).

In general, h2gb will have a single instance of [`Data`], which contains all
data that can be referenced. As of this writing, the data is all loaded into
memory at start, but I imagine that in the future we might decide to
load data opportunistically instead.

## Organization

[`Data`] contains entries for each type of object - [`Constants`],
[`Enums`], [`Bitmasks`], and [`Types`] (perhaps more in the future). Each
one is implemented as a [`DataEntry`] of its respective type.

Within each type of data, everything is grouped under a *namespace*, which
you can think of as roughly analogous to a directory. For example, you might
group all Terraria-related constants/enums/etc. under the `Terraria`
namespace. Pretty much every function in [`DataEntry`] requires a namespace,
and most will return an [`Err`] value if the namespace does not exist.

Within a namespace, all data is organized under a *name*. You can think of
a name as roughly analogous to a filename (and, in fact, when loading from
the filesystem, that's where the name comes from!)

The exact semantics of the name vary depending on the field type:

* [`Constants`] - Constant names represent a logical grouping of constants,
  where the names must be unique but values can be duplicated. I envision a
  group of constants to be like a single header-file's worth
* [`Enums`] - A named enum represents a single enum in, say, C, where the
  values are meaningfully related in some way (but not necessarily unique,
  it turns out!)
* [`Bitmasks`] - A name represents a single set of bits from a bitmask.
  These are unique within a name, though not necessarily exhaustive
* [`Types`] - the name is simply a name given to a single type

## Simple data types

For some data types (particularly [`Types`]), that's where it ends. In
fact, under the covers, [`Types`] is just an alias for [`H2Type`]. You can
use [`DataEntry::get`] to retrieve the actual type; for example:

```rust
use generic_number::*;
use h2datatype::*;
use h2datatype::data::*;
use h2datatype::simple::numeric::*;

// Create a Data structure
let mut data = Data::new();

// Load the type into this (you won't need to do this manually - it'll
// be loaded in a file or loaded at startup)
data.types.load_datum(
  Some("MyNamespace".to_string()), // Namespace
  "TypeName", // Name
  H2Integer::new(IntegerReader::I8, IntegerRenderer::default()), // H2Type
).unwrap();

// This is the important bit: get the type back out!
let t: &H2Type = data.types.get(
  Some("MyNamespace"),
  "TypeName",
).unwrap();

// ...do whatever you like with the H2Type
```

## Look-up-able types

Other types are effectively collections of name-value pairs. The difference
between constants, enums, and bitmasks is in particulars: how the data is
loaded, how it is queried, which option it supports, stuff like that. Those
implement the [`Lookupable`] trait, which lets [`DataTrait`] do helpful
lookups - specifically, [`DataTrait::lookup`] and
[`DataTrait::lookup_options`].

An example of looking up a constant:

```rust
use std::collections::HashMap;

use generic_number::*;
use h2datatype::*;
use h2datatype::data::*;
use h2datatype::simple::numeric::*;

// Create a Data structure
let mut data = Data::new();

// Create a set of constands - you won't have to do this by hand
let constants = Constants::load(&HashMap::from([
    ("name1".to_string(), "1".to_string()),
    ("name2".to_string(), "2".to_string()),
    ("name3".to_string(), "0x03".to_string()),
    ("name4".to_string(), "0b0100".to_string()),
])).unwrap();

// Load the constants - again, you won't need to do this directly
data.constants.load_datum(Some("MyNamespace".to_string()), "ConstantsName", constants);

// Look up a constant (this returns an array of possible values)
assert_eq!(vec!["name1".to_string()], data.constants.lookup(Some("MyNamespace"), "ConstantsName", 1).unwrap());
```

## File formats

Generally, data is stored on the filesystem and loaded from files. For
everything, we support JSON, YAML, and RON. Usually the parser is selected
by the file's extension, though the code loading it can specify.

We also support CSV for the simpler types - Constants, Enums, and Bitmaps.
But more complex types, such as Types, don't work.

TOML has similar problems to CSV, so I just opted to not support it at all.

Usually, when data is loaded, the filename (without extension) is the
_name_ of the type, and the parent folder name is the _namespace_. All
that can be configured by the loader as well, though.

License: MIT
