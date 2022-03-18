use std::fs::File;
use std::io;
use std::path::Path;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::Integer;

pub trait DataTrait : Sized {
    /// This is the data type as it's found in the file, not necessarily
    /// related to what the type actually is
    ///
    /// For example, a Constant is a `(String, String)` when it is serialized,
    /// but is a `(String, Integer)` when loaded.
    type SerializedType : DeserializeOwned + Serialize;

    /// Load the data from the type that was serialized.
    fn load(data: &Self::SerializedType) -> SimpleResult<Self>;

    /// (Optional) load from a vector of key/value pairs.
    ///
    /// Values can be blank (`None`) for certain types (such as Enum).
    ///
    /// Specifically used for CSV.
    fn load_str(_: Vec<(String, Option<Integer>)>) -> SimpleResult<Self> {
        bail!("Type cannot be loaded from a string");
    }

    /// Get the data in a format that can be serialized
    fn save(&self) -> SimpleResult<Self::SerializedType>;

    /// (Optional) export to a vector of key/value pairs.
    ///
    /// Specifically used for CSV.
    fn save_str(&self) -> SimpleResult<Vec<(String, Integer)>> {
        bail!("Type cannot be loaded from a string");
    }

    fn load_csv(reader: impl io::Read) -> SimpleResult<Self> {
        let mut out: Vec<(String, Option<Integer>)> = vec![];

        // Get ready to read the CSV
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(reader);

        // Loop through all the records
        for result in rdr.records() {
            let record = result.map_err(|e| {
                SimpleError::new(format!("Couldn't read CSV: {}", e))
            })?;

            // Get the first (the name) as a String
            let name = record.get(0).ok_or(
                SimpleError::new("Couldn't parse the CSV")
            )?.to_string();

            // Get the second (the value) and keep it as a String for now
            let value = match record.get(1) {
                Some(v) => Some(Integer::from_str(v)?),
                None => None,
            };

            // Insert it
            out.push((name, value))
        }

        Self::load_str(out)
    }

    fn load_from_csv_string(data: impl AsRef<str>) -> SimpleResult<Self> {
        Self::load_csv(data.as_ref().as_bytes())
    }

    fn load_from_csv_file(filename: &Path) -> SimpleResult<Self> {
        Self::load_csv(io::BufReader::new(File::open(filename).map_err(|e| {
            SimpleError::new(format!("Could not read file: {}", e))
        })?))
    }

    fn to_csv(&self) -> SimpleResult<String> {
        // Convert to String->String
        let mut w = csv::WriterBuilder::new().has_headers(false).from_writer(vec![]);

        for (name, value) in &self.save_str()? {
            w.write_record([name, &value.to_string()]).map_err(|e| {
                SimpleError::new(format!("Could not create CSV record: {:?}", e))
            })?;
        }

        let bytes = w.into_inner().map_err(|e| {
            SimpleError::new(format!("Couldn't write CSV: {:?}", e))
        })?;

        String::from_utf8(bytes).map_err(|e| {
            SimpleError::new(format!("Couldn't write CSV: {:?}", e))
        })
    }

    fn load_yaml(reader: impl io::Read) -> SimpleResult<Self> {
        // Initially read as String->String
        let d: Self::SerializedType = serde_yaml::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't parse YAML file: {:?}", e))
        })?;

        Self::load(&d)
    }

    fn load_from_yaml_string(data: impl AsRef<str>) -> SimpleResult<Self> {
        Self::load_yaml(data.as_ref().as_bytes())
    }

    fn load_from_yaml_file(filename: &Path) -> SimpleResult<Self> {
        Self::load_yaml(io::BufReader::new(File::open(filename).map_err(|e| {
            SimpleError::new(format!("Could not read file: {}", e))
        })?))
    }

    fn to_yaml(&self) -> SimpleResult<String> {
        serde_yaml::to_string(&self.save()?).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to YAML: {}", e))
        })
    }

    fn load_ron(reader: impl io::Read) -> SimpleResult<Self> {
        // Initially read as String->String
        let d: Self::SerializedType = ron::de::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't parse RON file: {:?}", e))
        })?;

        Self::load(&d)
    }

    fn load_from_ron_string(data: impl AsRef<str>) -> SimpleResult<Self> {
        Self::load_ron(data.as_ref().as_bytes())
    }

    fn load_from_ron_file(filename: &Path) -> SimpleResult<Self> {
        Self::load_ron(io::BufReader::new(File::open(filename).map_err(|e| {
            SimpleError::new(format!("Could not read file: {}", e))
        })?))
    }

    fn to_ron(&self) -> SimpleResult<String> {
        ron::to_string(&self.save()?).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to RON: {}", e))
        })
    }

    fn load_json(reader: impl io::Read) -> SimpleResult<Self> {
        // Initially read as String->String
        let d: Self::SerializedType = serde_json::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't parse YAML file: {:?}", e))
        })?;

        Self::load(&d)
    }

    fn load_from_json_string(data: impl AsRef<str>) -> SimpleResult<Self> {
        Self::load_json(data.as_ref().as_bytes())
    }

    fn load_from_json_file(filename: &Path) -> SimpleResult<Self> {
        Self::load_json(io::BufReader::new(File::open(filename).map_err(|e| {
            SimpleError::new(format!("Could not read file: {}", e))
        })?))
    }

    fn to_json(&self) -> SimpleResult<String> {
        serde_json::to_string_pretty(&self.save()?).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to JSON: {}", e))
        })
    }
}
