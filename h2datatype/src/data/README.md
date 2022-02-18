***Note: This file was automatically generated from [h2datatype/src/data/mod.rs](/h2datatype/src/data/mod.rs)***

Pre-canned datatypes for easier analysis.

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
