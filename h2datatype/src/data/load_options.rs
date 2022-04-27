#[derive(Debug)]
pub enum LoadNamespace {
    /// Load into the "root" namespace.
    ///
    /// This should only be used for built-in types
    None,

    /// Create the namespace based on the folder name, or no namespace.
    ///
    /// With nested folders and ambiguity of paths, this is probably not a
    /// great option.
    Auto,

    /// Load into a specific namespace.
    ///
    /// This is the most common option.
    Specific(String),
}

#[derive(Debug)]
pub enum LoadName {
    /// Base the name on the filename.
    ///
    /// This will usually be the correct choice.
    Auto,

    /// Use a specific name.
    ///
    /// This will fail when loading more than one file!
    Specific(String),
}

#[derive(Debug)]
pub struct LoadOptions {
    pub namespace: LoadNamespace,
    pub name: LoadName,
}

impl Default for LoadOptions {
    fn default() -> LoadOptions {
        Self {
            namespace: LoadNamespace::Auto,
            name: LoadName::Auto,
        }
    }
}

impl LoadOptions {
    pub fn new(namespace: LoadNamespace, name: LoadName) -> Self {
        Self {
            namespace,
            name,
        }
    }
}
