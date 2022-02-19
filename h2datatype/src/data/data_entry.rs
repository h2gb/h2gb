use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::path::Path;

use simple_error::{SimpleResult, bail};
use walkdir::WalkDir;

use crate::data::traits::*;
use crate::data::FileType;

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
#[derive(Debug)]
pub struct DataEntry<T: DataTrait> {
    pub data: HashMap<String, T>,
}

impl<T: DataTrait> Default for DataEntry<T> {
    fn default() -> DataEntry<T> {
        Self {
            data: HashMap::new()
        }
    }
}

impl<T: DataTrait> DataEntry<T> {
    /// Create a new, empty instance.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Get the name from the path.
    ///
    /// Returns the filename with no path and no extension.
    fn get_name(path: &Path) -> SimpleResult<String> {
        match path.file_stem() {
            Some(file) => Ok(file.to_string_lossy().to_string()),
            None => bail!("Could not read file name on path {:?}", path),
        }
    }

    /// Load data from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON, and RON (based on extension)
    pub fn load(&mut self, path: &Path) -> SimpleResult<&Self> {
        // This is kinda clunky, but it ensures that we don't have duplicates
        // within a set
        let mut duplicates: HashSet<String> = HashSet::new();

        // Catch invalid paths
        if !path.exists() {
            bail!("No such path: {:?}", path);
        }

        let thing: Vec<(String, T)> = WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
            .map(|e| {
                // I kept accidentally using the `path` argument
                let path = e.path();

                // Get the name with prefix
                let name = Self::get_name(path)?;

                // Check for duplicates within this group
                if duplicates.contains(&name) {
                    bail!("Duplicate name: {} (from path {:?})", name, path);
                }
                duplicates.insert(name.clone());

                // Load based on the extension
                let data = match FileType::from_filename(path) {
                    Some(FileType::YAML) => T::load_from_yaml_file(path)?,
                    Some(FileType::JSON) => T::load_from_json_file(path)?,
                    Some(FileType::RON)  => T::load_from_ron_file(path)?,
                    Some(FileType::CSV)  => T::load_from_csv_file(path)?,
                    None => bail!("Unrecognized file type: {:?}", path),
                };

                Ok((name, data))
            })
            .collect::<SimpleResult<Vec<(String, T)>>>()?;

        if let Err(e) = extend_no_duplicates(&mut self.data, thing) {
            bail!("Could not load data from {:?}: {}", path, e);
        }

        Ok(self)
    }

    /// Get the names of all available enums
    pub fn list(&self) -> Vec<&str> {
        self.data.keys().into_iter().map(|s| &s[..]).collect()
    }

    /// Find a specific value in an enum based on an [`Integer`].
    ///
    /// Empty list means no value was found, an `Err` is returned if the name does
    /// not exist.
    pub fn get(&self, name: &str) -> SimpleResult<&T> {
        match self.data.get(name) {
            Some(e) => Ok(e),
            None => bail!("Could not find {}", name),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.data.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T: DataTrait + Lookupable> DataEntry<T> {
    pub fn lookup(&self, name: &str, value: &T::LookupBy, options: T::LookupOptions) -> SimpleResult<T::LookupResult> {
        match self.data.get(name) {
            Some(e) => Ok(e.lookup(value, options)),
            None => bail!("Could not find {}", name),
        }
    }
}
