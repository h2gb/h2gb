use std::path::Path;

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
