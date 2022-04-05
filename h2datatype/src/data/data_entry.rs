use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::path::Path;

use simple_error::{SimpleResult, bail};
use walkdir::WalkDir;

use crate::data::traits::*;
use crate::data::{FileType, LoadOptions, LoadName, LoadNamespace};

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
    namespaces: HashMap<Option<String>, HashMap<String, T>>,
}

impl<T: DataTrait> Default for DataEntry<T> {
    fn default() -> DataEntry<T> {
        Self {
            namespaces: HashMap::new()
        }
    }
}

impl<T: DataTrait> DataEntry<T> {
    /// Create a new, empty instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load data from a [`Path`] (either a file or directory)
    ///
    /// Supports: YAML, CSV, JSON, and RON (based on extension)
    pub fn load(&mut self, path: &Path, options: &LoadOptions) -> SimpleResult<&Self> {
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

                // Get the name, based on settings
                let name = match &options.name {
                    LoadName::Auto => match path.file_stem() {
                        Some(file) => file.to_string_lossy().to_string(),
                        None => bail!("Could not read file name on path {:?}", path),
                    },

                    LoadName::Specific(s) => s.to_owned(),
                };

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

        // Figure out the namespace, based on the options
        let namespace = match &options.namespace {
            LoadNamespace::None => None,
            LoadNamespace::Auto => {
                todo!();
            },
            LoadNamespace::Specific(s) => Some(s.to_owned()),
        };

        // Get or insert the namespace
        let mut n = self.namespaces.entry(namespace).or_insert(HashMap::new());

        // Extend the list, but raise an error if there are duplicates
        // (This extend is atomic, in that it won't insert anything if one entry
        // is bad)
        if let Err(e) = extend_no_duplicates(&mut n, thing) {
            bail!("Could not load data from {:?}: {}", path, e);
        }

        Ok(self)
    }

    /// Get a list of all namespaces
    pub fn namespaces(&self) -> Vec<String> {
        self.namespaces
            .keys()                       // Get the keys
            .into_iter()                  // Convert to an iterator
            .filter(|x| x.is_some())      // Don't get the None namespace
            .map(|x| x.clone().unwrap())  // Convert the Option<String> to String (we know it's not None here)
            .collect()                    // Convert to Vec
    }

    fn namespace(&self, namespace: Option<&str>) -> SimpleResult<&HashMap<String, T>> {
        match self.namespaces.get(&namespace.map(|n| n.to_string())) {
            Some(n) => Ok(n),
            None => bail!("No such namespace: {:?}", namespace),
        }
    }

    /// Get the names of all available enums
    pub fn list(&self, namespace: Option<&str>) -> SimpleResult<Vec<&str>>
    {
        Ok(self.namespace(namespace)?
            .keys()
            .into_iter()
            .map(|s| s.as_ref())
            .collect())
    }

    /// Find a specific value based on its name + namespace
    ///
    /// Empty list means no value was found, an `Err` is returned if the name does
    /// not exist.
    pub fn get(&self, namespace: Option<&str>, name: impl AsRef<str>) -> SimpleResult<&T>
    {
        match self.namespace(namespace)?.get(name.as_ref()) {
            Some(t) => Ok(t),
            None => bail!("Could not find type {:?} in namespace {:?}"),
        }
    }

    pub fn contains(&self, namespace: Option<&str>, name: impl AsRef<str>) -> SimpleResult<bool> {
        Ok(self.namespace(namespace)?.contains_key(name.as_ref()))
    }

    pub fn len(&self, namespace: Option<&str>) -> usize {
        match self.namespace(namespace) {
            Ok(n) => n.len(),
            Err(_) => 0,
        }
    }
}

impl<T: DataTrait + Lookupable> DataEntry<T> {
    pub fn lookup(&self, namespace: Option<&str>, name: impl AsRef<str>, value: &T::LookupBy) -> SimpleResult<T::LookupResult> {
        match self.namespace(namespace)?.get(name.as_ref()) {
            Some(e) => Ok(e.lookup(value)),
            None => bail!("Could not find {}", name.as_ref()),
        }
    }

    pub fn lookup_options(&self, namespace: Option<&str>, name: impl AsRef<str>, value: &T::LookupBy, options: T::LookupOptions) -> SimpleResult<T::LookupResult> {
        match self.namespace(namespace)?.get(name.as_ref()) {
            Some(e) => Ok(e.lookup_options(value, options)),
            None => bail!("Could not find {}", name.as_ref()),
        }
    }
}
