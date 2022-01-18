use std::collections::{HashMap, HashSet};
use std::path::Path;

use simple_error::{SimpleResult, bail};
use walkdir::WalkDir;

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
        // TODO: We should have better error messages here
        let data = match Self::load(path, prefix) {
            Ok(d) => d,
            Err(e) => bail!("Could not load constants from {:?}: {}", path, e),
        };

        if let Err(e) = extend_no_duplicates(&mut self.constants, data) {
            bail!("Could not load constants from {:?}: {}", path, e);
        }

        Ok(self)
    }

    /// Load enums from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON (based on extension)
    pub fn load_enums(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<&Self> {
        let data = match Self::load(path, prefix) {
            Ok(d) => d,
            Err(e) => bail!("Could not load enums from {:?}: {}", path, e),
        };

        if let Err(e) = extend_no_duplicates(&mut self.enums, data) {
            bail!("Could not load enums from {:?}: {}", path, e);
        }

        Ok(self)
    }

    /// Load bitmasks from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON (based on extension)
    pub fn load_bitmasks(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<&Self> {
        let data = match Self::load(path, prefix) {
            Ok(d) => d,
            Err(e) => bail!("Could not load bitmasks from {:?}: {}", path, e),
        };

        if let Err(e) = extend_no_duplicates(&mut self.bitmasks, data) {
            bail!("Could not load bitmasks from {:?}: {}", path, e);
        }

        Ok(self)
    }

    /// Load types from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, JSON (based on extension) - does not support CSV
    pub fn load_types(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<&Self> {
        let data = match Self::load(path, prefix) {
            Ok(d) => d,
            Err(e) => bail!("Could not load types from {:?}: {}", path, e),
        };

        if let Err(e) = extend_no_duplicates(&mut self.types, data) {
            bail!("Could not load types from {:?}: {}", path, e);
        }

        Ok(self)
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
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/constants/test1.csv");
        let mut data = DataNg::new();

        data.load_constants(&d, None)?;

        // Make sure the output is sensible
        assert_eq!(1, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load a second file
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/constants/test2.json");
        data.load_constants(&d, None)?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Load an enum
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/enums/test1.csv");
        data.load_enums(&d, None)?;

        // Make sure the output is sensible
        assert_eq!(2, data.constants.len());
        assert_eq!(1, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Correctly error on bad filename
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/NOSUCHFILE");
        assert!(data.load_enums(&d, None).is_err());

        // Check a value
        assert_eq!(&Integer::from(100), data.constants.get("test1").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    #[test]
    fn test_load_directory() -> SimpleResult<()> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/constants/");
        let mut data = DataNg::new();

        data.load_constants(&d, None)?;

        // Make sure the output is sensible
        assert_eq!(3, data.constants.len());
        assert_eq!(0, data.enums.len());
        assert_eq!(0, data.bitmasks.len());
        assert_eq!(0, data.types.len());

        // Check a value
        assert_eq!(&Integer::from(100), data.constants.get("test1").unwrap().get_by_name("TEST2").unwrap());

        Ok(())
    }

    #[test]
    fn test_deeply_nested() -> SimpleResult<()> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/nested/");
        let mut data = DataNg::new();

        data.load_constants(&d, None)?;

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
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/constants/test1.csv");
        let mut data = DataNg::new();

        data.load_constants(&d, Some("MY_PREFIX"))?;

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
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/constants/test1.csv");
        let mut data = DataNg::new();

        // First time works, second time fails
        data.load_constants(&d, None)?;

        assert!(data.load_constants(&d, None).is_err());

        Ok(())
    }

    #[test]
    fn test_ambiguous_one_step() -> SimpleResult<()> {
        // Tests ambiguity from loading a directory with two different names
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/ambiguous");

        // Immediately fails
        assert!(DataNg::new().load_constants(&d, None).is_err());

        Ok(())
    }

    #[test]
    fn test_prefix_resolves_ambiguity() -> SimpleResult<()> {
        // Tests ambiguity from loading one, then loading a duplciate
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/constants/test1.csv");
        let mut data = DataNg::new();

        // First time works
        data.load_constants(&d, None)?;
        assert_eq!(1, data.constants.len());

        // Second time fails, when bare
        assert!(data.load_constants(&d, None).is_err());

        // Second time works, when we give it a name
        data.load_constants(&d, Some("MY_PREFIX"))?;
        assert_eq!(2, data.constants.len());

        Ok(())
    }

    #[test]
    fn test_prefix_resolves_ambiguity_directory() -> SimpleResult<()> {
        // Tests ambiguity from loading one, then loading a duplciate
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/constants/");
        let mut data = DataNg::new();

        // First time works
        data.load_constants(&d, None)?;
        assert_eq!(3, data.constants.len());

        // Second time fails, when bare
        assert!(data.load_constants(&d, None).is_err());

        // Second time works, when we give it a name
        data.load_constants(&d, Some("MY_PREFIX"))?;
        assert_eq!(6, data.constants.len());

        Ok(())
    }
}
