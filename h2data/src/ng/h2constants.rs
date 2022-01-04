use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use std::collections::HashMap;

use bimap::BiMap;
use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::Integer;

/// A named collection of constants
#[derive(Debug)]
pub struct H2Constants {
    constants: BiMap<String, Integer>,
}

impl H2Constants {
    pub fn new_empty() -> Self {
        Self {
            constants: BiMap::new(),
        }
    }

    fn read_csv<R>(reader: R) -> SimpleResult<BiMap<String, Integer>>
    where R: io::Read
    {
        let mut out = BiMap::new();

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader);

        for result in rdr.records() {
            let record = result.map_err(|e| {
                SimpleError::new(format!("Couldn't read CSV: {}", e))
            })?;

            if record.len() != 2 {
                bail!("CSV must be 2 records per line, this line was {}", record.len());
            }

            let name = record.get(0).ok_or(
                SimpleError::new("Couldn't parse the CSV")
            )?.to_string();

            let number: Integer = record.get(1).ok_or(
                SimpleError::new("Error reading the CSV file")
            )?.parse().map_err(|_| {
                SimpleError::new(format!("Couldn't parse second CSV field as integer"))
            })?;

            out.insert_no_overwrite(name, number).map_err(|e| {
                SimpleError::new(format!("Duplicate key or value while reading CSV: {:?}", e))
            })?;
        }

        Ok(out)
    }

    pub fn load_from_csv_string(data: &str) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_csv(data.as_bytes())?
        })
    }

    pub fn load_from_csv_file(filename: &PathBuf) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_csv(io::BufReader::new(File::open(filename).map_err(|e| {
                SimpleError::new(format!("Could not read file: {}", e))
            })?))?
        })
    }

    pub fn to_csv(&self) -> SimpleResult<String> {
        // Convert to String->String
        let mut w = csv::WriterBuilder::new().has_headers(false).from_writer(vec![]);

        for (k, v) in &self.constants {
            w.write_record(&[k.clone(), v.to_string()]).map_err(|e| {
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

    fn read_yaml<R>(reader: R) -> SimpleResult<BiMap<String, Integer>>
    where
        R: io::Read
    {
        // Read as String->String
        let h: HashMap<String, String> = serde_yaml::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't read YAML file as String->String mapping: {:?}", e))
        })?;

        // Convert to String->Integer
        let mut out: BiMap<String, Integer> = BiMap::new();
        for (k, v) in h.into_iter() {
            // Get the integer
            let i = Integer::from_str(&v).map_err(|e| {
                SimpleError::new(format!("Couldn't parse integer from YAML: {:?}", e))
            })?;

            out.insert_no_overwrite(k, i).map_err(|e| {
                SimpleError::new(format!("Duplicate key or value while reading YAML: {:?}", e))
            })?;
        }

        Ok(out)
    }

    pub fn load_from_yaml_string(data: &str) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_yaml(data.as_bytes())?
        })
    }

    pub fn load_from_yaml_file(filename: &PathBuf) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_yaml(io::BufReader::new(File::open(filename).map_err(|e| {
                SimpleError::new(format!("Could not read file: {}", e))
            })?))?
        })
    }

    pub fn to_yaml(&self) -> SimpleResult<String> {
        // Convert to String->String
        let mut h: HashMap<String, String> = HashMap::new();

        for (k, v) in &self.constants {
            h.insert(k.clone(), v.to_string());
        }

        serde_yaml::to_string(&h).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to YAML: {}", e))
        })
    }

    fn read_json<R>(reader: R) -> SimpleResult<BiMap<String, Integer>>
    where
        R: io::Read
    {
        // Read as String->String
        let h: HashMap<String, String> = serde_json::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't read JSON file as String->String mapping: {:?}", e))
        })?;

        // Convert to String->Integer
        let mut out: BiMap<String, Integer> = BiMap::new();
        for (k, v) in h.into_iter() {
            // Get the integer
            let i = Integer::from_str(&v).map_err(|e| {
                SimpleError::new(format!("Couldn't parse integer from JSON: {:?}", e))
            })?;

            out.insert_no_overwrite(k, i).map_err(|e| {
                SimpleError::new(format!("Duplicate key or value while reading JSON: {:?}", e))
            })?;
        }

        Ok(out)
    }

    pub fn load_from_json_string(data: &str) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_json(data.as_bytes())?
        })
    }

    pub fn load_from_json_file(filename: &PathBuf) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_json(io::BufReader::new(File::open(filename).map_err(|e| {
                SimpleError::new(format!("Could not read file: {}", e))
            })?))?
        })
    }

    pub fn to_json(&self) -> SimpleResult<String> {
        // Convert to String->String
        let mut h: HashMap<String, String> = HashMap::new();

        for (k, v) in &self.constants {
            h.insert(k.clone(), v.to_string());
        }

        serde_json::to_string_pretty(&h).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to JSON: {}", e))
        })
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Integer> {
        self.constants.get_by_left(name)
    }

    pub fn get_by_value(&self, value: &Integer) -> Option<&str> {
        self.constants.get_by_right(value).map(|v| &v[..])
    }

    pub fn len(&self) -> usize {
        self.constants.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_csv() -> SimpleResult<()> {
        // Most stuff works
        let constants: H2Constants = H2Constants::load_from_csv_string("TEST1,1\nTEST2,100\nTEST3,5\nTEST4,-10000\nTEST5,0x100\n")?;
        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        // Missing entries fails
        assert!(H2Constants::load_from_csv_string("TEST1,1\nTEST2\nTEST3,10\n").is_err());

        // Non-numbers fail
        assert!(H2Constants::load_from_csv_string("100,TEST1\n").is_err());

        // Blank lines are ignored
        assert_eq!(2, H2Constants::load_from_csv_string("TEST1,100\n\n\n\n\nTEST3,200\n")?.len());

        // Duplicates fail in both directions
        assert!(H2Constants::load_from_csv_string("TEST1,1\nTEST2,1\n").is_err());
        assert!(H2Constants::load_from_csv_string("TEST1,1\nTEST1,2\n").is_err());

        // Check if we can convert it back and forth
        let data = constants.to_csv()?;
        let constants = H2Constants::load_from_csv_string(&data)?;

        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_csv_file() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/test1.csv");

        let constants = H2Constants::load_from_csv_file(&d)?;

        // Do all the same tests as test_csv()
        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_csv_empty() -> SimpleResult<()> {
        assert_eq!(0, H2Constants::load_from_csv_string("")?.len());

        Ok(())
    }

    #[test]
    fn test_json() -> SimpleResult<()> {
        let data = "{ \"TEST1\": \"1\", \"TEST3\": \"5\", \"TEST2\": \"100\", \"TEST4\": \"-10000\", \"TEST5\": \"0x100\" }";
        let constants: H2Constants = H2Constants::load_from_json_string(data)?;

        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        // Check if we can convert it back and forth
        let data = constants.to_json()?;
        let constants = H2Constants::load_from_json_string(&data)?;

        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_json_file() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/test2.json");

        let constants = H2Constants::load_from_json_file(&d)?;

        // Do all the same tests as test_json()
        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_yaml() -> SimpleResult<()> {
        let data = "---
TEST4: -10000
TEST1: 1
TEST2: 100
TEST3: 5
TEST5: 256";

        let constants: H2Constants = H2Constants::load_from_yaml_string(data)?;

        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        let data = constants.to_yaml()?;
        let constants = H2Constants::load_from_yaml_string(&data)?;

        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_yaml_file() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/test3.yaml");

        let constants = H2Constants::load_from_yaml_file(&d)?;

        // Do all the same tests as test_yaml()
        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        Ok(())
    }
}
