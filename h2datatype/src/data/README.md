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

// Look up a constant
assert_eq!(vec!["name1".to_string()], data.constants.lookup(Some("MyNamespace"), "ConstantsName", &1.into()).unwrap());

```

XXX: This has gone way outta date

This module is a layer designed for accessing information that is stored on-
disk. Definitions of constants, lists of enums, stuff like that.

As of writing, we support the following datatypes:

* Constants - a group of named values, grouped under a single filename
* Enums - a group of named values, usually unique, and usually incremental values
* Bitmasks - a group of values that each represent a bit in an integer
* Types - a single datatype

With some limitations, they can be loaded from any of these file types:

* YAML
* JSON
* CSV
* RON - Rust Object Notation

Types cannot use CSV, and enums can only have incremental values
(automatically generated) in CSV format, since the other formats are
unordered.

### Loading

In general, you'll want a single instance of [`Data`] for the application,
to load data into it at startup, and to pass it around as needed.

To load initially, use the various load functions:

* [`Data::load_constants`]
* [`Data::load_enums`]
* [`Data::load_bitmasks`]
* [`Data::load_types`]

Those functions all take a [`&Path`] argument, which is the path to load.
That can either be a filename or a directory. If it's a directory, it will
recurse to find files. The extensions of the files determine how the file is
parsed.

The loaded data will be named based on the filename is it loaded from, which
must be unique.

The various `list_*` and `lookup_*` functions can be used to retrieve data.

License: MIT
