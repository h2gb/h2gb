use std::fs::File;
use std::io;
use std::path::PathBuf;

use simple_error::{SimpleResult, SimpleError};

use crate::H2Type;

#[derive(Debug)]
pub struct Types {
    h2type: H2Type,
}

impl Types {
    fn load_yaml<R>(reader: R) -> SimpleResult<Self>
    where
        R: io::Read
    {
        // Initially read as String->String
        let t: H2Type = serde_yaml::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't read YAML file as an H2Type object: {}", e))
        })?;

        Ok(Self {
            h2type: t,
        })
    }

    pub fn load_from_yaml_string(data: &str) -> SimpleResult<Self> {
        Self::load_yaml(data.as_bytes())
    }

    pub fn load_from_yaml_file(filename: &PathBuf) -> SimpleResult<Self> {
        Self::load_yaml(io::BufReader::new(File::open(filename).map_err(|e| {
            SimpleError::new(format!("Could not read file: {}", e))
        })?))
    }

    pub fn to_yaml(&self) -> SimpleResult<String> {
        serde_yaml::to_string(&self.h2type).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to YAML: {}", e))
        })
    }

    fn load_json<R>(reader: R) -> SimpleResult<Self>
    where
        R: io::Read
    {
        // Initially read as String->String
        let t: H2Type = serde_json::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't read JSON file as an H2Type object: {}", e))
        })?;

        Ok(Self {
            h2type: t,
        })
    }

    pub fn load_from_json_string(data: &str) -> SimpleResult<Self> {
        Self::load_json(data.as_bytes())
    }

    pub fn load_from_json_file(filename: &PathBuf) -> SimpleResult<Self> {
        Self::load_json(io::BufReader::new(File::open(filename).map_err(|e| {
            SimpleError::new(format!("Could not read file: {}", e))
        })?))
    }

    pub fn to_json(&self) -> SimpleResult<String> {
        serde_json::to_string_pretty(&self.h2type).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to JSON: {}", e))
        })
    }

    pub fn get(&self) -> &H2Type {
        &self.h2type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use generic_number::*;

    #[test]
    fn test_json_file() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/types/struct.json");
        let constants = Types::load_from_json_file(&d)?;

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        let resolved = constants.get().resolve(Context::new(&data), None)?;
        assert_eq!(2, resolved.children.len());
        assert_eq!(Integer::from(0x04030201u32), resolved.children.get(0).unwrap().as_integer.unwrap());
        assert_eq!(Integer::from(0xddccbbaau32), resolved.children.get(1).unwrap().as_integer.unwrap());

        Ok(())
    }

    #[test]
    fn test_yaml_file() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/types/struct.yaml");
        let constants = Types::load_from_yaml_file(&d)?;

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        let resolved = constants.get().resolve(Context::new(&data), None)?;
        assert_eq!(2, resolved.children.len());
        assert_eq!(Integer::from(0x04030201u32), resolved.children.get(0).unwrap().as_integer.unwrap());
        assert_eq!(Integer::from(0xddccbbaau32), resolved.children.get(1).unwrap().as_integer.unwrap());

        Ok(())
    }
}
