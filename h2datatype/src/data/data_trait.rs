use std::fs::File;
use std::io;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use simple_error::{SimpleResult, SimpleError};

pub trait DataTrait : Sized {
    /// This is the data type as it's found in the file, not necessarily
    /// related to what the type actually is
    ///
    /// For example, a Constant is a `(String, String)` when it is serialized,
    /// but is a `(String, Integer)` when loaded.
    type SerializedType : DeserializeOwned + Serialize;

    /// Load the data from the type that was serialized.
    fn load(data: &Self::SerializedType) -> SimpleResult<Self>;

    /// Get the data in a format that can be serialized
    fn save(&self) -> SimpleResult<Self::SerializedType>;

    // fn load_csv<R>(&mut self, reader: R) -> SimpleResult<()>
    // where
    //     R: io::Read;

    // fn load_from_csv_string(&mut self, data: &str) -> SimpleResult<()> {
    //     self.load_csv(data.as_bytes())
    // }

    // fn load_from_csv_file(&mut self, filename: &PathBuf) -> SimpleResult<()> {
    //     self.load_csv(io::BufReader::new(File::open(filename).map_err(|e| {
    //         SimpleError::new(format!("Could not read file: {}", e))
    //     })?))
    // }

    // fn to_csv(&self) -> SimpleResult<String>;

    fn load_yaml<R>(reader: R) -> SimpleResult<Self>
    where
        R: io::Read
    {
        // Initially read as String->String
        let d: Self::SerializedType = serde_yaml::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't parse YAML file: {:?}", e))
        })?;

        Self::load(&d)
    }

    fn load_from_yaml_string(data: &str) -> SimpleResult<Self> {
        Self::load_yaml(data.as_bytes())
    }

    fn load_from_yaml_file(filename: &PathBuf) -> SimpleResult<Self> {
        Self::load_yaml(io::BufReader::new(File::open(filename).map_err(|e| {
            SimpleError::new(format!("Could not read file: {}", e))
        })?))
    }

    fn to_yaml(&self) -> SimpleResult<String> {
        serde_yaml::to_string(&self.save()?).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to YAML: {}", e))
        })
    }

    fn load_json<R>(reader: R) -> SimpleResult<Self>
    where
        R: io::Read
    {
        // Initially read as String->String
        let d: Self::SerializedType = serde_json::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't parse YAML file: {:?}", e))
        })?;

        Self::load(&d)
    }

    fn load_from_json_string(data: &str) -> SimpleResult<Self> {
        Self::load_json(data.as_bytes())
    }

    fn load_from_json_file(filename: &PathBuf) -> SimpleResult<Self> {
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
