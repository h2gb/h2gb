//! Pre-canned datatypes for easier analysis.
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
use std::path::Path;

use simple_error::{SimpleResult, bail};

use generic_number::{Integer, IntegerRenderer};

use crate::H2Type;

mod classes;
use classes::*;

mod traits;
use traits::*;

mod data_entry;
use data_entry::DataEntry;

mod file_type;
use file_type::FileType;

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

/// The core [`Data`] struct, which holds all data data that has been loaded.
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
            constants: DataEntry::new(),
            enums:     DataEntry::new(),
            bitmasks:  DataEntry::new(),
            types:     DataEntry::new(),
        }
    }

    /// Load constants from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON, and RON (based on extension)
    pub fn load_constants(&mut self, path: &Path) -> SimpleResult<&Self> {
        self.constants.load(path)?;

        Ok(self)
    }

    /// Load enums from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON, and RON (based on extension)
    pub fn load_enums(&mut self, path: &Path) -> SimpleResult<&Self> {
        self.enums.load(path)?;

        Ok(self)
    }

    /// Load bitmasks from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON, and RON (based on extension)
    pub fn load_bitmasks(&mut self, path: &Path) -> SimpleResult<&Self> {
        self.bitmasks.load(path)?;

        Ok(self)
    }

    /// Load types from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, JSON, and RON (based on extension) - does not support
    /// CSV
    pub fn load_types(&mut self, path: &Path) -> SimpleResult<&Self> {
        self.types.load(path)?;

        Ok(self)
    }

    /// Get the names of all available enums
    pub fn list_enums(&self) -> Vec<&str> {
        self.enums.list()
    }

    /// Find a specific value in an enum based on an [`Integer`].
    ///
    /// Empty list means no value was found, an `Err` is returned if the name does
    /// not exist.
    pub fn lookup_enum(&self, enum_name: &str, value: &Integer) -> SimpleResult<Vec<String>> {
        self.enums.lookup(enum_name, value, ())
    }

    /// Get the names of all available bitmasks
    pub fn list_bitmasks(&self) -> Vec<&str> {
        self.bitmasks.list()
    }

    /// Find a specific bitmask matches based on an [`Integer`].
    ///
    /// An optional `unknown_renderer` can be supplied, which will be used to
    /// render unknown values using the (prefix, renderer) tuple.
    ///
    /// Additionally, "negative" matches can be included. That means that the
    /// output will look like `X | Y | ~Z`)
    pub fn lookup_bitmask(&self, bitmask_name: &str, value: &Integer, unknown_renderer: Option<(String, IntegerRenderer)>, show_negatives: bool) -> SimpleResult<Vec<String>> {
        let options: BitmaskOptions = BitmaskOptions::new(unknown_renderer, show_negatives);

        self.bitmasks.lookup(bitmask_name, value, Some(options))
    }

    /// Get the names of all available groups of constants
    pub fn list_constant_groups(&self) -> Vec<&str> {
        self.constants.list()
    }

    /// Find a specific constant or constants based on an [`Integer`].
    ///
    /// Empty list means no value was found, an `Err` is returned if the name does
    /// not exist.
    pub fn lookup_constant(&self, constant_group: &str, value: &Integer) -> SimpleResult<Vec<String>> {
        self.enums.lookup(constant_group, value, ())
    }

    /// Get the names of all available types.
    pub fn list_types(&self) -> Vec<&str> {
        self.types.list()
    }

    /// Find a specific type by name.
    pub fn get_type(&self, type_name: &str) -> SimpleResult<&H2Type> {
        Ok(self.types.get(type_name)?.get())
    }

    // /// Add a new type to the list.
    // pub fn insert_type(&mut self, type_name: &str, h2type: &H2Type) -> SimpleResult<()> {
    //     if self.types.contains_key(type_name) {
    //         bail!("Duplicate type: {}", type_name);
    //     }

    //     self.types.insert(type_name.to_string(), Types::from(h2type));

    //     Ok(())
    // }
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
        data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load a second file
        data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test2.json"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load an enum
        data.load_enums(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/test1.csv"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len());
        assert_eq!(1, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load a .ron file
        data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test4.ron"].iter().collect::<PathBuf>())?;
        assert_eq!(3, data.constants.len());
        assert_eq!(1, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Correctly error on bad filename
        assert!(data.load_enums(&[env!("CARGO_MANIFEST_DIR"), "testdata/NOSUCHFILE"].iter().collect::<PathBuf>()).is_err());

        // Check a value
        assert_eq!(&Integer::from(100), data.constants.get("test1").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    #[test]
    fn test_load_directory() -> SimpleResult<()> {
        let mut data = Data::new();
        data.load_enums(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(0, data.constants.len());
        assert_eq!(3, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Check the names
        let mut e = data.list_enums();
        e.sort();
        assert_eq!(vec!["test1", "test2", "test3"], e);

        // Retrieve a value
        assert_eq!(vec!["TEST2".to_string()], data.lookup_enum("test1", &Integer::from(100))?);

        Ok(())
    }

    #[test]
    fn test_deeply_nested() -> SimpleResult<()> {
        let mut data = Data::new();
        data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/nested"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Check a value
        assert_eq!(&Integer::from(100), data.constants.get("constants").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    // #[test]
    // fn test_prefix() -> SimpleResult<()> {
    //     let mut data = Data::new();
    //     data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>(), Some("MY_PREFIX"))?;

    //     // Make sure the output is sensible
    //     assert_eq!(1, data.constants.len());
    //     assert_eq!(0, data.enums.len());
    //     assert_eq!(0, data.bitmasks.len());
    //     assert_eq!(0, data.types.len());

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
        data.load_constants(&path)?;
        assert!(data.load_constants(&path).is_err());

        Ok(())
    }

    #[test]
    fn test_ambiguous_one_step() -> SimpleResult<()> {
        // Immediately fails
        assert!(Data::new().load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/ambiguous"].iter().collect::<PathBuf>()).is_err());

        Ok(())
    }

    // #[test]
    // fn test_prefix_resolves_ambiguity() -> SimpleResult<()> {
    //     // Tests ambiguity from loading one, then loading a duplicate
    //     let mut data = Data::new();
    //     let path = [env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>();

    //     // First time works
    //     data.load_constants(&path, None)?;
    //     assert_eq!(1, data.constants.len());

    //     // Second time fails, when bare
    //     assert!(data.load_constants(&path, None).is_err());

    //     // Second time works, when we give it a name
    //     data.load_constants(&path, Some("MY_PREFIX"))?;
    //     assert_eq!(2, data.constants.len());

    //     Ok(())
    // }

    // #[test]
    // fn test_prefix_resolves_ambiguity_directory() -> SimpleResult<()> {
    //     // Tests ambiguity from loading one, then loading a duplciate
    //     let mut data = Data::new();
    //     let path = [env!("CARGO_MANIFEST_DIR"), "testdata/constants"].iter().collect::<PathBuf>();

    //     // First time works
    //     data.load_constants(&path, None)?;
    //     assert_eq!(4, data.constants.len());

    //     // Second time fails, when bare
    //     assert!(data.load_constants(&path, None).is_err());

    //     // Second time works, when we give it a name
    //     data.load_constants(&path, Some("MY_PREFIX"))?;
    //     assert_eq!(8, data.constants.len());

    //     Ok(())
    // }
}
