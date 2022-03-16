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


// Braining..... how do namespaces work?
//
// Goal:
// * Data has a namespace of some sort for organization + conflicts
// * Enums, Bitmasks, types, etc. are loaded into a namespace
//   * Specified by the user, or by the file?
// * By default, a type searches for data within its own namespace
//   * So when it's loaded, the namespace is set
//   * Namespace is in h2type, but not serialized (?)
//

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use simple_error::{SimpleResult, bail};

mod classes;
use classes::{Bitmasks, Enums, Types, Constants};

mod traits;

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

/// The struct that holds data. Not designed to be serialized or stored.
///
/// TODO: write more about Data
/// This structure is recursive - data can be nested to any level, or none.
#[derive(Default, Debug)]
pub struct Data {
    namespaces: HashMap<String, Data>,

    pub constants: DataEntry<Constants>,
    pub enums:     DataEntry<Enums>,
    pub bitmasks:  DataEntry<Bitmasks>,
    pub types:     DataEntry<Types>,
}

impl Data {
    /// Create a new, empty instance.
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::default(),

            constants: DataEntry::default(),
            enums:     DataEntry::default(),
            bitmasks:  DataEntry::default(),
            types:     DataEntry::default(),
        }
    }

    pub fn namespace(&self, name: Option<&str>) -> SimpleResult<&Data> {
        match name {
            Some(n) => {
                match self.namespaces.get(&n[..]) {
                    Some(n) => Ok(n),
                    None    => bail!("No such namespace: {:?}", name),
                }
            },
            None => Ok(self),
        }
    }

    pub fn namespace_mut(&mut self, name: Option<&str>) -> SimpleResult<&mut Data> {
        match name {
            Some(n) => {
                match self.namespaces.get_mut(&n[..]) {
                    Some(n) => Ok(n),
                    None    => bail!("No such namespace: {:?}", name),
                }
            },
            None => Ok(&mut self),
        }
    }

    pub fn namespace_create(&mut self, name: &str) -> SimpleResult<()> {
        if self.namespaces.contains_key(name) {
            bail!("Namespace already exists: {}", name);
        }

        self.namespaces.insert(name.to_string(), Data::default());

        Ok(())
    }

    /// Get a list of all namespaces.
    pub fn namespaces(&self) -> Vec<&String> {
        self.namespaces.keys().collect()
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
        data.constants.load(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load a second file
        data.constants.load(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test2.json"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load an enum
        data.enums.load(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/test1.csv"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len());
        assert_eq!(1, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load a .ron file
        data.constants.load(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test4.ron"].iter().collect::<PathBuf>())?;
        assert_eq!(3, data.constants.len());
        assert_eq!(1, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Correctly error on bad filename
        assert!(data.enums.load(&[env!("CARGO_MANIFEST_DIR"), "testdata/NOSUCHFILE"].iter().collect::<PathBuf>()).is_err());

        // Check a value
        assert_eq!(&Integer::from(100u32), data.constants.get("test1").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    #[test]
    fn test_load_directory() -> SimpleResult<()> {
        let mut data = Data::new();
        data.enums.load(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(0, data.constants.len());
        assert_eq!(3, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Check the names
        let mut e = data.enums.list();
        e.sort();
        assert_eq!(vec!["test1", "test2", "test3"], e);

        // Retrieve a value
        assert_eq!(vec!["TEST2".to_string()], data.enums.lookup("test1", &Integer::from(100))?);

        Ok(())
    }

    #[test]
    fn test_deeply_nested() -> SimpleResult<()> {
        let mut data = Data::new();
        data.constants.load(&[env!("CARGO_MANIFEST_DIR"), "testdata/nested"].iter().collect::<PathBuf>())?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Check a value
        assert_eq!(&Integer::from(100u32), data.constants.get("constants").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    // #[test]
    // fn test_namespace() -> SimpleResult<()> { XXX TEST THIS
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
        data.constants.load(&path)?;
        assert!(data.constants.load(&path).is_err());

        Ok(())
    }

    #[test]
    fn test_ambiguous_one_step() -> SimpleResult<()> {
        // Immediately fails
        assert!(Data::new().constants.load(&[env!("CARGO_MANIFEST_DIR"), "testdata/ambiguous"].iter().collect::<PathBuf>()).is_err());

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
