use std::collections::{HashMap, HashSet};
use std::path::Path;

use simple_error::{SimpleResult, bail};
use walkdir::WalkDir;

use generic_number::Integer;

use crate::H2Type;

mod constants;
use constants::*;

mod enums;
use enums::*;

mod bitmasks;
use bitmasks::*;

mod types;
use types::*;

mod data_trait;
use data_trait::*;

/// Extend a [`HashMap`] without allowing duplicates.
fn extend_no_duplicates<T>(orig: &mut HashMap<String, T>, new: Vec<(String, T)>) -> SimpleResult<()> {
    // Loop through to ensure no duplicates
    let new = new.into_iter().map(|(key, value)| {
        match orig.contains_key(&key) {
            true => bail!("Duplicate key: {}", key),
            false => Ok((key, value))
        }
    }).collect::<SimpleResult<HashMap<String, T>>>()?;

    orig.extend(new);

    Ok(())
}

enum FileType {
    YAML,
    JSON,
    CSV,
}

impl FileType {
    fn from_filename(name: &Path) -> Option<Self> {
        let extension = name.extension()?.to_string_lossy().to_string();

        match &extension[..] {
            "yaml" => Some(Self::YAML),
            "yml"  => Some(Self::YAML),
            "json" => Some(Self::JSON),
            "csv"  => Some(Self::CSV),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct DataNg {
    pub constants: HashMap<String, Constants>,
    pub enums:     HashMap<String, Enums>,
    pub bitmasks:  HashMap<String, Bitmasks>,
    pub types:     HashMap<String, Types>,
}

impl DataNg {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            enums:     HashMap::new(),
            bitmasks:  HashMap::new(),
            types:     HashMap::new(),
        }
    }

    fn get_name(path: &Path, prefix: Option<&str>) -> SimpleResult<String> {
        let file = match path.file_stem() {
            Some(file) => {
                file.to_string_lossy().to_string()
            }
            None => bail!("Could not read file name on path {:?}", path),
        };

        match prefix {
            Some(p) => Ok(format!("{}::{}", p, file)),
            None => Ok(file),
        }
    }

    /// Internal function to load any [`DataTrait`] type.
    fn load<T: DataTrait>(path: &Path, prefix: Option<&str>) -> SimpleResult<Vec<(String, T)>> {
        // This is kinda clunky, but it ensures that we don't have duplicates
        // within a set
        let mut duplicates: HashSet<String> = HashSet::new();

        // Catch invalid paths
        if !path.exists() {
            bail!("No such path: {:?}", path);
        }

        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
            .map(|e| {
                // I kept accidentally using the `path` argument
                let path = e.path();

                // Get the name with prefix
                let name = Self::get_name(path, prefix)?;

                // Check for duplicates within this group
                if duplicates.contains(&name) {
                    bail!("Duplicate name: {} (from path {:?})", name, path);
                }
                duplicates.insert(name.clone());

                // Load based on the extension
                let data = match FileType::from_filename(path) {
                    Some(FileType::YAML) => T::load_from_yaml_file(path)?,
                    Some(FileType::JSON) => T::load_from_json_file(path)?,
                    Some(FileType::CSV)  => T::load_from_csv_file(path)?,
                    None => bail!("Unrecognized file type: {:?}", path),
                };

                Ok((name, data))
            })
            .collect()
    }

    /// Load constants from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON (based on extension)
    pub fn load_constants(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<&Self> {
        // TODO: We should bubble up better error messages
        if let Err(e) = extend_no_duplicates(&mut self.constants, Self::load(path, prefix)?) {
            bail!("Could not load constants from {:?}: {}", path, e);
        }

        Ok(self)
    }

    /// Load enums from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON (based on extension)
    pub fn load_enums(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<&Self> {
        if let Err(e) = extend_no_duplicates(&mut self.enums, Self::load(path, prefix)?) {
            bail!("Could not load enums from {:?}: {}", path, e);
        }

        Ok(self)
    }

    /// Load bitmasks from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON (based on extension)
    pub fn load_bitmasks(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<&Self> {
        if let Err(e) = extend_no_duplicates(&mut self.bitmasks, Self::load(path, prefix)?) {
            bail!("Could not load enums from {:?}: {}", path, e);
        }

        Ok(self)
    }

    /// Load types from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, JSON (based on extension) - does not support CSV
    pub fn load_types(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<&Self> {
        if let Err(e) = extend_no_duplicates(&mut self.types, Self::load(path, prefix)?) {
            bail!("Could not load enums from {:?}: {}", path, e);
        }

        Ok(self)
    }

    pub fn list_enums(&self) -> Vec<&str> {
        self.enums.keys().into_iter().map(|s| &s[..]).collect()
    }

    pub fn lookup_enum(&self, enum_name: &str, value: &Integer) -> SimpleResult<Vec<String>> {
        match self.enums.get(enum_name) {
            Some(e) => Ok(e.get_by_value(value)),
            None => bail!("No such enum: {}", enum_name),
        }
    }

    pub fn list_bitmasks(&self) -> Vec<&str> {
        self.bitmasks.keys().into_iter().map(|s| &s[..]).collect()
    }

    pub fn lookup_bitmask(&self, bitmask_name: &str, value: &Integer) -> SimpleResult<Vec<String>> {
        match self.bitmasks.get(bitmask_name) {
            Some(e) => Ok(e.get_by_value(value)),
            None => bail!("No such bitmask: {}", bitmask_name),
        }
    }

    pub fn list_constant_groups(&self) -> Vec<&str> {
        self.constants.keys().into_iter().map(|s| &s[..]).collect()
    }

    pub fn lookup_constant(&self, constant_group: &str, value: &Integer) -> SimpleResult<Vec<String>> {
        match self.constants.get(constant_group) {
            Some(e) => Ok(e.get_by_value(value)),
            None => bail!("No such constant: {}", constant_group),
        }
    }

    pub fn list_types(&self) -> Vec<&str> {
        self.types.keys().into_iter().map(|s| &s[..]).collect()
    }

    pub fn lookup_type(&self, type_name: &str) -> SimpleResult<&H2Type> {
        match self.types.get(type_name) {
            Some(t) => Ok(t.get()),
            None => bail!("No such type: {}", type_name),
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

        let mut data = DataNg::new();
        data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>(), None)?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load a second file
        data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test2.json"].iter().collect::<PathBuf>(), None)?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load an enum
        data.load_enums(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/test1.csv"].iter().collect::<PathBuf>(), None)?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len());
        assert_eq!(1, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Correctly error on bad filename
        assert!(data.load_enums(&[env!("CARGO_MANIFEST_DIR"), "testdata/NOSUCHFILE"].iter().collect::<PathBuf>(), None).is_err());

        // Check a value
        assert_eq!(&Integer::from(100), data.constants.get("test1").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    #[test]
    fn test_load_directory() -> SimpleResult<()> {
        let mut data = DataNg::new();
        data.load_enums(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/"].iter().collect::<PathBuf>(), None)?;

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
        let mut data = DataNg::new();
        data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/nested"].iter().collect::<PathBuf>(), None)?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Check a value
        assert_eq!(&Integer::from(100), data.constants.get("constants").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    #[test]
    fn test_prefix() -> SimpleResult<()> {
        let mut data = DataNg::new();
        data.load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>(), Some("MY_PREFIX"))?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Check a value
        assert_eq!(&Integer::from(100), data.constants.get("MY_PREFIX::test1").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    #[test]
    fn test_ambiguous_two_steps() -> SimpleResult<()> {
        // Tests ambiguity from loading one, then loading a duplciate
        let mut data = DataNg::new();

        let path = [env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>();

        // Works the first time, not the second
        data.load_constants(&path, None)?;
        assert!(data.load_constants(&path, None).is_err());

        Ok(())
    }

    #[test]
    fn test_ambiguous_one_step() -> SimpleResult<()> {
        // Immediately fails
        assert!(DataNg::new().load_constants(&[env!("CARGO_MANIFEST_DIR"), "testdata/ambiguous"].iter().collect::<PathBuf>(), None).is_err());

        Ok(())
    }

    #[test]
    fn test_prefix_resolves_ambiguity() -> SimpleResult<()> {
        // Tests ambiguity from loading one, then loading a duplicate
        let mut data = DataNg::new();
        let path = [env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>();

        // First time works
        data.load_constants(&path, None)?;
        assert_eq!(1, data.constants.len());

        // Second time fails, when bare
        assert!(data.load_constants(&path, None).is_err());

        // Second time works, when we give it a name
        data.load_constants(&path, Some("MY_PREFIX"))?;
        assert_eq!(2, data.constants.len());

        Ok(())
    }

    #[test]
    fn test_prefix_resolves_ambiguity_directory() -> SimpleResult<()> {
        // Tests ambiguity from loading one, then loading a duplciate
        let mut data = DataNg::new();
        let path = [env!("CARGO_MANIFEST_DIR"), "testdata/constants"].iter().collect::<PathBuf>();

        // First time works
        data.load_constants(&path, None)?;
        assert_eq!(3, data.constants.len());

        // Second time fails, when bare
        assert!(data.load_constants(&path, None).is_err());

        // Second time works, when we give it a name
        data.load_constants(&path, Some("MY_PREFIX"))?;
        assert_eq!(6, data.constants.len());

        Ok(())
    }
}
