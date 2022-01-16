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

#[derive(Debug)]
pub struct DataNg {
    constants: HashMap<String, Constants>,
    enums:     HashMap<String, Enums>,
    bitmasks:  HashMap<String, Bitmasks>,
    types:     HashMap<String, Types>,
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

    fn load_file<T: DataTrait>(path: &Path) -> SimpleResult<T> {
        match FileType::from_filename(path) {
            Some(FileType::YAML) => T::load_from_yaml_file(path),
            Some(FileType::JSON) => T::load_from_json_file(path),
            Some(FileType::CSV)  => T::load_from_csv_file(path),
            None => bail!("Unrecognized file type: {:?}", path),
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


    fn load_directory<T: DataTrait>(path: &Path, prefix: Option<&str>) -> SimpleResult<Vec<(String, T)>> {
        // This is kinda clunky, but it ensures that we don't have duplicates
        // within a set
        let mut duplicates: HashSet<String> = HashSet::new();

        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
            .map(|e| {
                let name = Self::get_name(e.path(), prefix)?;

                // Check for duplicates
                if duplicates.contains(&name) {
                    bail!("Duplicate name: {} (from path {:?})", name, path);
                }
                duplicates.insert(name.clone());

                let data = Self::load_file(e.path())?;

                Ok((name, data))
            })
            .collect()
    }

    fn load_constants_dir(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<()> {
        extend_no_duplicates(&mut self.constants, Self::load_directory(path, prefix)?)
    }

    fn load_enums_dir(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<()> {
        extend_no_duplicates(&mut self.enums, Self::load_directory(path, prefix)?)
    }

    fn load_bitmasks_dir(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<()> {
        extend_no_duplicates(&mut self.bitmasks, Self::load_directory(path, prefix)?)
    }

    fn load_types_dir(&mut self, path: &Path, prefix: Option<&str>) -> SimpleResult<()> {
        extend_no_duplicates(&mut self.types, Self::load_directory(path, prefix)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_load_directory() -> SimpleResult<()> {
        let mut data = DataNg::new();

        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/constants/");
        data.load_constants_dir(&d, None)?;

        Ok(())
    }

    #[test]
    fn test_load_multiple_steps() -> SimpleResult<()> {
        todo!()
    }

    #[test]
    fn test_nested() -> SimpleResult<()> {
        todo!()
    }

    #[test]
    fn test_ambiguous() -> SimpleResult<()> {
        todo!()
    }
}
