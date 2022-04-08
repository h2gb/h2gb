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
/// Mostly, this is a pretty thin wrapper that adds namespaces, and passes most
/// requests through to the internal trait functions - [`DataTrait`] and
/// [`Lookupable`] (and the modules that implement them) do most of the actual
/// effort.
///
/// The [`None`] namespace will always exist.
#[derive(Debug)]
pub struct DataEntry<T: DataTrait> {
    namespaces: HashMap<Option<String>, HashMap<String, T>>,
}

impl<T: DataTrait> Default for DataEntry<T> {
    fn default() -> DataEntry<T> {
        let mut empty_field: HashMap<_, _> = HashMap::new();
        empty_field.insert(None, HashMap::new());

        Self {
            namespaces: empty_field
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
    /// can't be parsed, or if any duplicate names end up existing within a
    /// namespace. We try to raise errors before actually making changes - the
    /// data *should* be consistent (ie, nothing loaded) if an error is raised.
    ///
    /// Namespace will be opportunistically created if needed. Existing
    /// namespaces can be used as well (based on the [`LoadOptions`]), as long
    /// as there are no naming conflicts within.
    ///
    /// Supports: YAML, CSV, JSON, and RON (based on extension)
    pub fn load_path(&mut self, path: &Path, options: &LoadOptions) -> SimpleResult<()> {
        // This is kinda clunky, but it ensures that we don't have duplicates
        // within a set
        let mut duplicates: HashSet<(Option<String>, String)> = HashSet::new();

        // Catch invalid paths
        if !path.exists() {
            bail!("No such path: {:?}", path);
        }

        let thing: Vec<(Option<String>, (String, T))> = WalkDir::new(path)
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

                // Get the namespace, also based on settings
                let namespace = match &options.namespace {
                    LoadNamespace::None => None,
                    LoadNamespace::Auto => {
                        match path.parent() {
                            Some(p) => match p.file_stem() {
                                Some(p) => Some(p.to_string_lossy().to_string()),
                                None => bail!("Could not read parent name on path {:?}", path),
                            },
                            None => bail!("Could not read parent name on path {:?}", path),
                        }
                    },
                    LoadNamespace::Specific(s) => Some(s.to_owned()),
                };

                // Check for duplicates within this group
                if duplicates.contains(&(namespace.clone(), name.clone())) {
                    bail!("Duplicate name within set: {:?}::{} (from path {:?})", namespace, name, path);
                }
                duplicates.insert((namespace.clone(), name.clone()));

                // Check if the namespace already exists
                if let Ok(n) = self.namespace(namespace.as_deref()) {
                    // If the namespace exists, check if it already contains the key - that's bad!
                    if n.contains_key(&name) {
                        bail!("Duplicate name from previous data: {:?}::{} (from path {:?})", namespace, name, path);
                    }
                }

                // Load based on the extension
                let data = match FileType::from_filename(path) {
                    Some(FileType::YAML) => T::load_from_yaml_file(path)?,
                    Some(FileType::JSON) => T::load_from_json_file(path)?,
                    Some(FileType::RON)  => T::load_from_ron_file(path)?,
                    Some(FileType::CSV)  => T::load_from_csv_file(path)?,
                    None => bail!("Unrecognized file type: {:?}", path),
                };

                Ok((namespace, (name, data)))
            })
            .collect::<SimpleResult<Vec<(Option<String>, (String, T))>>>()?;

        // If we've gotten here, we know we're free of duplicates and can load the data
        for (namespace, data) in thing.into_iter() {
            self.load_data(namespace, vec![data])?;
        }

        Ok(())
    }

    /// Get a list of all namespaces.
    ///
    /// Will not return the default [`None`] namespace.
    pub fn list_namespaces(&self) -> Vec<String> {
        self.namespaces
            .keys()                       // Get the keys
            .into_iter()                  // Convert to an iterator
            .filter(|x| x.is_some())      // Don't get the None namespace
            .map(|x| x.clone().unwrap())  // Convert the Option<String> to String (we know it's not None here)
            .collect()                    // Convert to Vec
    }

    /// Internal function to get a namespace
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

    /// Find a specific set of values based on its name + namespace.
    ///
    /// Check out the functions in the [`Lookupable`] trait before using this
    /// one! They simplify reading from a datatype that is composed of key/value
    /// pairs.
    pub fn get(&self, namespace: Option<&str>, name: impl AsRef<str>) -> SimpleResult<&T>
    {
        match self.namespace(namespace)?.get(name.as_ref()) {
            Some(t) => Ok(t),
            None => bail!("Could not find type {:?} in namespace {:?}"),
        }
    }

    /// Does the namespace exist?
    pub fn contains_namespace(&self, namespace: Option<&str>) -> bool {
        self.namespace(namespace).is_ok()
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

    pub fn export_single_entry(&self, namespace: Option<&str>, name: impl AsRef<str>, file_type: FileType) -> SimpleResult<String> {
        let out = match self.namespace(namespace)?.get(name.as_ref()) {
            Some(t) => t,
            None => bail!("Could not find type {:?} in namespace {:?}"),
        };

        match file_type {
            FileType::CSV  => out.to_csv(),
            FileType::RON  => out.to_ron(),
            FileType::JSON => out.to_json(),
            FileType::YAML => out.to_yaml(),
        }
    }
}

impl<T: DataTrait + Lookupable> DataEntry<T> {
    pub fn lookup(&self, namespace: Option<&str>, name: impl AsRef<str>, value: impl Into<T::LookupBy>) -> SimpleResult<T::LookupResult> {
        match self.namespace(namespace)?.get(name.as_ref()) {
            Some(e) => Ok(e.lookup(value)),
            None => bail!("Could not find {}", name.as_ref()),
        }
    }

    pub fn lookup_options(&self, namespace: Option<&str>, name: impl AsRef<str>, value: impl Into<T::LookupBy>, options: T::LookupOptions) -> SimpleResult<T::LookupResult> {
        match self.namespace(namespace)?.get(name.as_ref()) {
            Some(e) => Ok(e.lookup_options(value, options)),
            None => bail!("Could not find {}", name.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use crate::data::Constants;

    #[test]
    fn test_load_empty_datum() -> SimpleResult<()> {
        let mut d: DataEntry<Constants> = DataEntry::default();

        // No namespaces
        assert_eq!(0, d.list_namespaces().len());

        // Load a piece of data
        d.load_datum(Some("MyNamespace".into()), "name", Constants::default())?;

        // Namespace auto-created
        assert_eq!(1, d.list_namespaces().len());

        // Make sure it's sensible
        assert_eq!(1, d.len(Some("MyNamespace")));
        assert_eq!(0, d.len(Some("NoSuchNamespace")));
        assert_eq!(0, d.len(None));

        // Make sure it fails if we try again
        assert!(d.load_datum(Some("MyNamespace".into()), "name", Constants::default()).is_err());

        // Still only one namespace
        assert_eq!(1, d.list_namespaces().len());

        // Make sure it succeeds if we change the namespace
        assert!(d.load_datum(Some("MyNamespace2".into()), "name", Constants::default()).is_ok());

        // New namespace created
        assert_eq!(2, d.list_namespaces().len());

        // Try to use contains()
        assert_eq!(false, d.contains(Some("MyNamespace"), "anything")?);
        assert!(d.contains(Some("NoSuchNamespace"), "anything").is_err());

        // Make sure the list of fields in MyNamespace are correct
        assert_eq!(vec!["name"], d.list(Some("MyNamespace"))?);

        // We can't list a fake namesapce
        assert!(d.list(Some("NoSuchNamespace")).is_err());

        // The default namespace should always work
        assert_eq!(0, d.list(None)?.len());

        Ok(())
    }

    #[test]
    fn test_load_actual_data() -> SimpleResult<()> {
        let c1data = Constants::load(&HashMap::from([
            ("field1".to_string(), "1".to_string()),
            ("field2".to_string(), "2".to_string()),
        ]))?;

        let c2data = Constants::load(&HashMap::from([
            ("field3".to_string(), "3".to_string()),
            ("field4".to_string(), "4".to_string()),
        ]))?;

        let default_data = Constants::load(&HashMap::from([
            ("field0".to_string(), "0".to_string()),
            ("field-1".to_string(), "-1".to_string()),
        ]))?;

        let mut d: DataEntry<Constants> = DataEntry::default();

        // Load data into one namespace
        d.load_data(Some("MyNamespace".into()), vec![("name1".to_string(), c1data), ("name2".to_string(), c2data)])?;

        // Load data into the None namespace
        d.load_data(None, vec![("name1".to_string(), default_data)])?;

        // Ensure it's all sensible
        assert_eq!(2, d.len(Some("MyNamespace")));
        assert_eq!(0, d.len(Some("NoSuchNamespace")));
        assert_eq!(1, d.len(None));

        // Test a normal value
        assert_eq!(vec!["field1"], d.lookup(Some("MyNamespace"), "name1", 1u32)?);

        // Test a bad value
        assert_eq!(Vec::<&str>::new(), d.lookup(Some("MyNamespace"), "name1", 10000u32)?);

        // Ensure empty namespace = no value
        assert_eq!(Vec::<&str>::new(), d.lookup(None, "name1", 1u32)?);

        // Ensure non-existent namespace = error
        assert!(d.lookup(Some("NoSuchNamespace"), "name1", 1u32).is_err());

        // Test lengths
        assert_eq!(1, d.list_namespaces().len());
        assert_eq!(2, d.list(Some("MyNamespace"))?.len());
        assert_eq!(1, d.list(None)?.len());

        //println!("\n\n{}\n\n", d.export_single_entry(Some("MyNamespace"), "name1", FileType::CSV).unwrap());

        Ok(())
    }

}
