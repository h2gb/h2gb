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

/// [`DataEntry`] is the core struct that holds a single type of data.
///
/// A single type of data can be any class that implements [`DataTrait`]. In
/// practice, at the time of this writing, there are four kinds: Enums,
/// Bitmasks, Constants, and Types. Since many of the datatypes (all but Types)
/// are basically key-value pairs, we have extra support for types that
/// implement [`Lookupable`], to skip a step and perform a lookup.
///
/// All data is optionally namespaced. That is, the data is organized by
/// a "namespace", which helps group things and prevent conflicts. Mostly
/// these functions raise an error if trying to access a non-existent namespace.
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

    /// Load a set of data into a namespace.
    ///
    /// The namespace will automatically be created if it does not already
    /// exist. Any duplicate names within the namespace will return an error
    /// and nothing will be inserted (the insert is atomic).
    pub fn load_data(&mut self, namespace: Option<String>, data: impl Into<Vec<(String, T)>>) -> SimpleResult<()> {
        // Get or insert the namespace
        let mut n = self.namespaces.entry(namespace).or_insert(HashMap::new());

        // Extend the list, but raise an error if there are duplicates
        // (This extend is atomic, in that it won't insert anything if one entry
        // is bad)
        if let Err(e) = extend_no_duplicates(&mut n, data.into()) {
            bail!("Could not load data: {}", e);
        }

        Ok(())
    }

    /// Load a single instance of data into a namespace.
    ///
    /// The namespace with automatically be created if it does not already
    /// exist. If the name already exists, an error is returned.
    pub fn load_datum(&mut self, namespace: Option<String>, name: impl Into<String>, data: impl Into<T>) -> SimpleResult<()> {
        // Just off-load to the load_data function
        self.load_data(namespace, vec![(name.into(), data.into())])
    }

    /// Load data from a [`Path`] (either a file or directory).
    ///
    /// Special options (such as which namespace and how to name the values)
    /// are stored in the `options` argument. See the [`LoadOptions`] definition
    /// for all that.
    ///
    /// This will return an error if the path doesn't exist, if any of the files
    /// can't be parsed, or if any duplicate names end up existing. We try to
    /// raise errors before actually making changes - the data *should* be
    /// consistent (ie, nothing loaded) if an error is raised.
    ///
    /// Supports: YAML, CSV, JSON, and RON (based on extension)
    pub fn load_path(&mut self, path: &Path, options: &LoadOptions) -> SimpleResult<()> {
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
                // XXX: This needs an implementation
                todo!();
            },
            LoadNamespace::Specific(s) => Some(s.to_owned()),
        };

        self.load_data(namespace, thing)
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

    /// Get the names of all available names in the namespace.
    ///
    /// Returns an `Err` if the namespace does not exist.
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

    /// Does the namespace contain the given name?
    ///
    /// Returns an `Err` if the namespace does not exist.
    pub fn contains(&self, namespace: Option<&str>, name: impl AsRef<str>) -> SimpleResult<bool> {
        Ok(self.namespace(namespace)?.contains_key(name.as_ref()))
    }

    /// How many entries exist in this namespace?
    ///
    /// Returns `0` if the namespace does not exist.
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
