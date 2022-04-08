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
//! // Look up a constant (this returns an array of possible values)
//! assert_eq!(vec!["name1".to_string()], data.constants.lookup(Some("MyNamespace"), "ConstantsName", 1).unwrap());
//! ```

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
        assert_eq!(vec!["TEST2".to_string()], data.enums.lookup(None, "test1", 100)?);

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

    #[test]
    fn test_auto_namespace() -> SimpleResult<()> {
        let mut data = Data::new();
        let path = [env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>();

        // Loading with an automatic namespace should create a namespace named after the folder, and a set of constants names after the file
        data.constants.load_path(&path, &LoadOptions::new(LoadNamespace::Auto, LoadName::Auto))?;
        assert_eq!(1, data.constants.list_namespaces().len());
        assert_eq!(vec!["constants".to_string()], data.constants.list_namespaces());
        assert_eq!(vec!["TEST1".to_string()], data.constants.lookup(Some("constants"), "test1", 1)?);

        Ok(())
    }

    #[test]
    fn test_load_ambiguity() -> SimpleResult<()> {
        // Tests ambiguity from loading one, then loading a duplciate
        let mut data = Data::new();
        let path = [env!("CARGO_MANIFEST_DIR"), "testdata/otherambiguous"].iter().collect::<PathBuf>();

        // If we load the /otherambiguous/ folder into a set namespace, it will
        // fail because the two files have the same name
        assert!(data.constants.load_path(&path, &LoadOptions::new(LoadNamespace::Specific("namespace".to_string()), LoadName::Auto)).is_err());

        // Make sure nothing loaded
        assert_eq!(0, data.constants.list_namespaces().len());

        // Now automatically generate namespaces
        assert!(data.constants.load_path(&path, &LoadOptions::new(LoadNamespace::Auto, LoadName::Auto)).is_ok());

        // Now there should be two namespaces
        assert_eq!(2, data.constants.list_namespaces().len());

        // Here, we have "namespace1" and "namespace2". Load another set of
        // files that contains "namespace0" and "namespace1" - neither should
        // load
        let path = [env!("CARGO_MANIFEST_DIR"), "testdata/otherotherambiguous"].iter().collect::<PathBuf>();
        assert!(data.constants.load_path(&path, &LoadOptions::new(LoadNamespace::Specific("namespace".to_string()), LoadName::Auto)).is_err());

        // Nothing should have changed
        assert_eq!(2, data.constants.list_namespaces().len());

        Ok(())
    }
}
