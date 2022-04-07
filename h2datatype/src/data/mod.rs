//! Pre-canned datatypes for easier analysis.
//!
//! This document will focus on usage from the perspective of somebody writing
//! a module. How data gets into this originally is outside the scope (check
//! out the documentation for [`DataEntry`] if you're interested).
//!
//! In general, h2gb will have a single instance of [`Data`], which contains all
//! data that can be referenced. As of this writing, the data is all loaded into
//! memory at start, but I imagine that in the future we might decide to
//! load data opportunistically instead.
//!
//! # Organization
//!
//! [`Data`] contains entries for each type of object - [`Constants`],
//! [`Enums`], [`Bitmasks`], and [`Types`] (perhaps more in the future). Each
//! one is implemented as a [`DataEntry`] of its respective type.
//!
//! Within each type of data, everything is grouped under a *namespace*, which
//! you can think of as roughly analogous to a directory. For example, you might
//! group all Terraria-related constants/enums/etc. under the `Terraria`
//! namespace. Pretty much every function in [`DataEntry`] requires a namespace,
//! and most will return an [`Err`] value if the namespace does not exist.
//!
//! Within a namespace, all data is organized under a *name*. You can think of
//! a name as roughly analogous to a filename (and, in fact, when loading from
//! the filesystem, that's where the name comes from!)
//!
//! The exact semantics of the name vary depending on the field type:
//!
//! * [`Constants`] - Constant names represent a logical grouping of constants,
//!   where the names must be unique but values can be duplicated. I envision a
//!   group of constants to be like a single header-file's worth
//! * [`Enums`] - A named enum represents a single enum in, say, C, where the
//!   values are meaningfully related in some way (but not necessarily unique,
//!   it turns out!)
//! * [`Bitmasks`] - A name represents a single set of bits from a bitmask.
//!   These are unique within a name, though not necessarily exhaustive
//! * [`Types`] - the name is simply a name given to a single type
//!
//! # Simple data types
//!
//! For some data types (particularly [`Types`]), that's where it ends. In
//! fact, under the covers, [`Types`] is just an alias for [`H2Type`]. You can
//! use [`DataEntry::get`] to retrieve the actual type; for example:
//!
//! ```
//! use generic_number::*;
//! use h2datatype::*;
//! use h2datatype::data::*;
//! use h2datatype::simple::numeric::*;
//!
//! // Create a Data structure
//! let mut data = Data::new();
//!
//! // Load the type into this (you won't need to do this manually - it'll
//! // be loaded in a file or loaded at startup)
//! data.types.load_datum(
//!   Some("MyNamespace".to_string()), // Namespace
//!   "TypeName", // Name
//!   H2Integer::new(IntegerReader::I8, IntegerRenderer::default()), // H2Type
//! ).unwrap();
//!
//! // This is the important bit: get the type back out!
//! let t: &H2Type = data.types.get(
//!   Some("MyNamespace"),
//!   "TypeName",
//! ).unwrap();
//!
//! // ...do whatever you like with the H2Type
//! ```
//!
//! # Look-up-able types
//!
//! Other types are effectively collections of name-value pairs. The difference
//! between constants, enums, and bitmasks is in particulars: how the data is
//! loaded, how it is queried, which option it supports, stuff like that. Those
//! implement the [`Lookupable`] trait, which lets [`DataTrait`] do helpful
//! lookups - specifically, [`DataTrait::lookup`] and
//! [`DataTrait::lookup_options`].
//!
//! An example of looking up a constant:
//!
//! ```
//! use std::collections::HashMap;
//!
//! use generic_number::*;
//! use h2datatype::*;
//! use h2datatype::data::*;
//! use h2datatype::simple::numeric::*;
//!
//! // Create a Data structure
//! let mut data = Data::new();
//!
//! // Create a set of constands - you won't have to do this by hand
//! let constants = Constants::load(&HashMap::from([
//!     ("name1".to_string(), "1".to_string()),
//!     ("name2".to_string(), "2".to_string()),
//!     ("name3".to_string(), "0x03".to_string()),
//!     ("name4".to_string(), "0b0100".to_string()),
//! ])).unwrap();
//!
//! // Load the constants - again, you won't need to do this directly
//! data.constants.load_datum(Some("MyNamespace".to_string()), "ConstantsName", constants);
//!
//! // Look up a constant
//! assert_eq!(vec!["name1".to_string()], data.constants.lookup(Some("MyNamespace"), "ConstantsName", &1.into()).unwrap());
//!
//! ```
//!
//! XXX: This has gone way outta date
//!
//! This module is a layer designed for accessing information that is stored on-
//! disk. Definitions of constants, lists of enums, stuff like that.
//!
//! As of writing, we support the following datatypes:
//!
//! * Constants - a group of named values, grouped under a single filename
//! * Enums - a group of named values, usually unique, and usually incremental values
//! * Bitmasks - a group of values that each represent a bit in an integer
//! * Types - a single datatype
//!
//! With some limitations, they can be loaded from any of these file types:
//!
//! * YAML
//! * JSON
//! * CSV
//! * RON - Rust Object Notation
//!
//! Types cannot use CSV, and enums can only have incremental values
//! (automatically generated) in CSV format, since the other formats are
//! unordered.
//!
//! ## Loading
//!
//! In general, you'll want a single instance of [`Data`] for the application,
//! to load data into it at startup, and to pass it around as needed.
//!
//! To load initially, use the various load functions:
//!
//! * [`Data::load_constants`]
//! * [`Data::load_enums`]
//! * [`Data::load_bitmasks`]
//! * [`Data::load_types`]
//!
//! Those functions all take a [`&Path`] argument, which is the path to load.
//! That can either be a filename or a directory. If it's a directory, it will
//! recurse to find files. The extensions of the files determine how the file is
//! parsed.
//!
//! The loaded data will be named based on the filename is it loaded from, which
//! must be unique.
//!
//! The various `list_*` and `lookup_*` functions can be used to retrieve data.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use simple_error::{SimpleResult, bail};

mod classes;
pub use classes::*;

mod traits;
pub use traits::*;

mod data_entry;
pub use data_entry::*;

mod file_type;
use file_type::*;

mod load_options;
pub use load_options::*;

/// Extend a [`HashMap`] without allowing duplicates.
fn extend_no_duplicates<K, T>(orig: &mut HashMap<K, T>, new: Vec<(K, T)>) -> SimpleResult<()>
where K: Eq + Hash + Debug
{
    // Loop through to ensure no duplicates
    let new = new.into_iter().map(|(key, value)| {
        match orig.contains_key(&key) {
            true => bail!("Duplicate key: {:?}", key),
            false => Ok((key, value))
        }
    }).collect::<SimpleResult<HashMap<K, T>>>()?;

    orig.extend(new);

    Ok(())
}

/// Holds a variety of different datatypes during program execution.
///
/// This is designed to be initialized when the program executes, stored exactly
/// once in memory (due to size), and passed around as an immutable reference.
///
/// I can conceive of a future where this loads data opportunistically, but
/// that's an optimization I don't plan to do unless we need it.
///
/// To use any of the fields, access the internal [`DataEntry`] fields directly.
#[derive(Default, Debug)]
pub struct Data {
    pub constants: DataEntry<Constants>,
    pub enums:     DataEntry<Enums>,
    pub bitmasks:  DataEntry<Bitmasks>,
    pub types:     DataEntry<Types>,
}

impl Data {
    /// Create a new, empty instance.
    pub fn new() -> Self {
        Self {
            constants: DataEntry::default(),
            enums:     DataEntry::default(),
            bitmasks:  DataEntry::default(),
            types:     DataEntry::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use generic_number::Integer;

    #[test]
    fn test_load_file() -> SimpleResult<()> {
        let mut data = Data::new();
        data.constants.load_path(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>(), &LoadOptions::new(LoadNamespace::None, LoadName::Auto))?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len(None));
        assert_eq!(0, data.enums.len(None));
        assert_eq!(0, data.bitmasks.len(None));
        assert_eq!(0, data.types.len(None));

        // Load a second file
        data.constants.load_path(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test2.json"].iter().collect::<PathBuf>(), &LoadOptions::new(LoadNamespace::None, LoadName::Auto))?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len(None));
        assert_eq!(0, data.enums.len(None));
        assert_eq!(0, data.bitmasks.len(None));
        assert_eq!(0, data.types.len(None));

        // Load an enum
        data.enums.load_path(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/test1.csv"].iter().collect::<PathBuf>(), &LoadOptions::new(LoadNamespace::None, LoadName::Auto))?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len(None));
        assert_eq!(1, data.enums.len(None));
        assert_eq!(0, data.bitmasks.len(None));
        assert_eq!(0, data.types.len(None));

        // Load a .ron file
        data.constants.load_path(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test4.ron"].iter().collect::<PathBuf>(), &LoadOptions::new(LoadNamespace::None, LoadName::Auto))?;
        assert_eq!(3, data.constants.len(None));
        assert_eq!(1, data.enums.len(None));
        assert_eq!(0, data.bitmasks.len(None));
        assert_eq!(0, data.types.len(None));

        // Correctly error on bad filename
        assert!(data.enums.load_path(&[env!("CARGO_MANIFEST_DIR"), "testdata/NOSUCHFILE"].iter().collect::<PathBuf>(), &LoadOptions::new(LoadNamespace::None, LoadName::Auto)).is_err());

        // Check a value
        assert_eq!(&Integer::from(100u32), data.constants.get(None, "test1").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    #[test]
    fn test_load_directory() -> SimpleResult<()> {
        let mut data = Data::new();
        data.enums.load_path(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/"].iter().collect::<PathBuf>(), &LoadOptions::new(LoadNamespace::None, LoadName::Auto))?;

        // Make sure the output is sensible
        assert_eq!(0, data.constants.len(None));
        assert_eq!(3, data.enums.len(None));
        assert_eq!(0, data.bitmasks.len(None));
        assert_eq!(0, data.types.len(None));

        // Check the names
        let mut e = data.enums.list(None)?;
        e.sort();
        assert_eq!(vec!["test1", "test2", "test3"], e);

        // Retrieve a value
        assert_eq!(vec!["TEST2".to_string()], data.enums.lookup(None, "test1", &Integer::from(100u32))?);

        Ok(())
    }

    #[test]
    fn test_deeply_nested() -> SimpleResult<()> {
        let mut data = Data::new();
        data.constants.load_path(&[env!("CARGO_MANIFEST_DIR"), "testdata/nested"].iter().collect::<PathBuf>(), &LoadOptions::new(LoadNamespace::None, LoadName::Auto))?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len(None));
        assert_eq!(0, data.enums.len(None));
        assert_eq!(0, data.bitmasks.len(None));
        assert_eq!(0, data.types.len(None));

        // Check a value
        assert_eq!(&Integer::from(100u32), data.constants.get(None, "constants").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    // #[test]
    // fn test_namespace() -> SimpleResult<()> { XXX TEST THIS
    //     let mut data = Data::new();
    //     data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>(), Some("MY_PREFIX"))?;

    //     // Make sure the output is sensible
    //     assert_eq!(1, data.constants.len(None)?);
    //     assert_eq!(0, data.enums.len(None)?);
    //     assert_eq!(0, data.bitmasks.len(None)?);
    //     assert_eq!(0, data.types.len(None)?);

    //     // Check a value
    //     assert_eq!(&Integer::from(100), data.constants.get("MY_PREFIX::test1").unwrap().get_by_name("TEST2").unwrap());

    //     Ok(())
    // }

    #[test]
    fn test_ambiguous_two_steps() -> SimpleResult<()> {
        // Tests ambiguity from loading one, then loading a duplciate
        let mut data = Data::new();

        let path = [env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>();

        // Works the first time, not the second
        data.constants.load_path(&path, &LoadOptions::new(LoadNamespace::None, LoadName::Auto))?;
        assert!(data.constants.load_path(&path, &LoadOptions::new(LoadNamespace::None, LoadName::Auto)).is_err());

        Ok(())
    }

    #[test]
    fn test_ambiguous_one_step() -> SimpleResult<()> {
        // Immediately fails
        assert!(Data::new().constants.load_path(&[env!("CARGO_MANIFEST_DIR"), "testdata/ambiguous"].iter().collect::<PathBuf>(), &LoadOptions::new(LoadNamespace::None, LoadName::Auto)).is_err());

        Ok(())
    }

    // #[test]
    // fn test_prefix_resolves_ambiguity() -> SimpleResult<()> {
    //     // Tests ambiguity from loading one, then loading a duplicate
    //     let mut data = Data::new();
    //     let path = [env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>();

    //     // First time works
    //     data.load_constants(&path, None)?;
    //     assert_eq!(1, data.constants.len(None)?);

    //     // Second time fails, when bare
    //     assert!(data.load_constants(&path, None).is_err());

    //     // Second time works, when we give it a name
    //     data.load_constants(&path, Some("MY_PREFIX"))?;
    //     assert_eq!(2, data.constants.len(None)?);

    //     Ok(())
    // }

    // #[test]
    // fn test_prefix_resolves_ambiguity_directory() -> SimpleResult<()> {
    //     // Tests ambiguity from loading one, then loading a duplciate
    //     let mut data = Data::new();
    //     let path = [env!("CARGO_MANIFEST_DIR"), "testdata/constants"].iter().collect::<PathBuf>();

    //     // First time works
    //     data.load_constants(&path, None)?;
    //     assert_eq!(4, data.constants.len(None)?);

    //     // Second time fails, when bare
    //     assert!(data.load_constants(&path, None).is_err());

    //     // Second time works, when we give it a name
    //     data.load_constants(&path, Some("MY_PREFIX"))?;
    //     assert_eq!(8, data.constants.len(None)?);

    //     Ok(())
    // }
}
