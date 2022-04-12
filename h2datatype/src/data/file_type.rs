use std::path::Path;

/// The filetypes that h2gb can read data from.
///
/// I tried to support the most useful formats. There are limitations, though.
///
/// CSV works, but can only store textual data.
///
/// I attempted TOML support, but ultimately I don't think it's better than
/// CSV since it can't serialize non-string values for us
/// ([ref](https://stackoverflow.com/questions/57560593/why-do-i-get-an-unsupportedtype-error-when-serializing-to-toml-with-a-manually-i)).
pub enum FileType {
    YAML,
    JSON,
    CSV,
    RON,
}

impl FileType {
    pub fn from_filename(name: &Path) -> Option<Self> {
        let extension = name.extension()?.to_string_lossy().to_string();

        match &extension[..] {
            "yaml" => Some(Self::YAML),
            "yml"  => Some(Self::YAML),
            "json" => Some(Self::JSON),
            "csv"  => Some(Self::CSV),
            "ron"  => Some(Self::RON),
            _ => None,
        }
    }
}
