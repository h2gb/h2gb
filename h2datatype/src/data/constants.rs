use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use std::collections::HashMap;

use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::Integer;

/// A named collection of constants, fetched by name or value.
///
/// Constants in h2gb are simply name->value pairs, where within each set of
/// constants, the names are unique but values may not be.
///
/// Constants can be fetched either by name or by value. Fetching by value
/// returns a list of names associated with it (and that's probably the most
/// common usecase - looking up constants).
///
/// These values can be loaded from a variety of file formats, and are parsed
/// into [`generic_number::Integer`] instances. The formats are all, ultimately,
/// key value pairs, where both the key (name of constant) and value (the value
/// of the constant) are strings, which are parsed into the best fitting
/// `Integer`. We also support prefixes (like `0x` for hex, `0o` for octal,
/// etc).
#[derive(Debug)]
pub struct Constants {
    by_name: HashMap<String, Integer>,
    by_value: HashMap<Integer, Vec<String>>,
}

impl Constants {
    fn new_empty() -> Self {
        Self {
            by_name: HashMap::new(),
            by_value: HashMap::new(),
        }
    }

    fn add_entry(&mut self, name: &str, value: Integer) -> SimpleResult<()> {
        // Check for duplicate names
        if self.by_name.contains_key(name) {
            bail!("Duplicate constant value: {}", name);
        }

        // Insert
        self.by_name.insert(name.to_string(), value);

        // Insert or append to the by_value map
        let e = self.by_value.entry(value).or_insert(vec![]);
        e.push(name.to_string());

        Ok(())
    }

    fn load_csv<R>(reader: R) -> SimpleResult<Self>
    where R: io::Read
    {
        let mut out = Self::new_empty();

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader);

        for result in rdr.records() {
            let record = result.map_err(|e| {
                SimpleError::new(format!("Couldn't read CSV: {}", e))
            })?;

            // Ensure that there are only two entries per line
            if record.len() != 2 {
                bail!("CSV must be 2 records per line, this line was {}", record.len());
            }

            // Get the first (the name) as a String
            let name = record.get(0).ok_or(
                SimpleError::new("Couldn't parse the CSV")
            )?.to_string();

            // Get the second (the value) as an Integer
            let value: Integer = record.get(1).ok_or(
                SimpleError::new("Error reading the CSV file")
            )?.parse().map_err(|_| {
                SimpleError::new(format!("Couldn't parse second CSV field as integer"))
            })?;

            // Insert it
            out.add_entry(&name, value)?;
        }

        Ok(out)
    }

    pub fn load_from_csv_string(data: &str) -> SimpleResult<Self> {
        Self::load_csv(data.as_bytes())
    }

    pub fn load_from_csv_file(filename: &PathBuf) -> SimpleResult<Self> {
        Self::load_csv(io::BufReader::new(File::open(filename).map_err(|e| {
            SimpleError::new(format!("Could not read file: {}", e))
        })?))
    }

    pub fn to_csv(&self) -> SimpleResult<String> {
        // Convert to String->String
        let mut w = csv::WriterBuilder::new().has_headers(false).from_writer(vec![]);

        for (name, value) in &self.by_name {
            w.write_record(&[name.clone(), value.to_string()]).map_err(|e| {
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

    fn load_yaml<R>(reader: R) -> SimpleResult<Self>
    where
        R: io::Read
    {
        // Initially read as String->String
        let h: HashMap<String, String> = serde_yaml::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't read YAML file as String->String mapping: {:?}", e))
        })?;

        // Convert to String->Integer
        let mut out = Self::new_empty();
        for (name, value) in h.into_iter() {
            // Get the integer
            let value = Integer::from_str(&value).map_err(|e| {
                SimpleError::new(format!("Couldn't parse integer from YAML: {:?}", e))
            })?;

            out.add_entry(&name, value)?;
        }

        Ok(out)
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
        // Convert to String->String
        let mut h: HashMap<String, String> = HashMap::new();

        for (k, v) in &self.by_name {
            h.insert(k.clone(), v.to_string());
        }

        serde_yaml::to_string(&h).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to YAML: {}", e))
        })
    }

    fn load_json<R>(reader: R) -> SimpleResult<Self>
    where
        R: io::Read
    {
        // Read as String->String
        let h: HashMap<String, String> = serde_json::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't read JSON file as String->String mapping: {:?}", e))
        })?;

        // Convert to String->Integer
        let mut out = Self::new_empty();
        for (name, value) in h.into_iter() {
            // Get the integer
            let value = Integer::from_str(&value).map_err(|e| {
                SimpleError::new(format!("Couldn't parse integer from JSON: {:?}", e))
            })?;

            out.add_entry(&name, value)?;
        }

        Ok(out)
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
        // Convert to String->String
        let mut h: HashMap<String, String> = HashMap::new();

        for (k, v) in &self.by_name {
            h.insert(k.clone(), v.to_string());
        }

        serde_json::to_string_pretty(&h).map_err(|e| {
            SimpleError::new(format!("Failed to serialize to JSON: {}", e))
        })
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Integer> {
        self.by_name.get(name)
    }

    pub fn get_by_value(&self, value: &Integer) -> Option<&Vec<String>> {
        self.by_value.get(value)
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
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
        let constants: Constants = Constants::load_from_csv_string("TEST1,1\nTEST2,100\nTEST3,5\nTEST4,-10000\nTEST5,0x100\n")?;
        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        // Missing entries fails
        assert!(Constants::load_from_csv_string("TEST1,1\nTEST2\nTEST3,10\n").is_err());

        // Non-numbers fail
        assert!(Constants::load_from_csv_string("100,TEST1\n").is_err());

        // Blank lines are ignored
        assert_eq!(2, Constants::load_from_csv_string("TEST1,100\n\n\n\n\nTEST3,200\n")?.len());

        // Duplicate names fail
        assert!(Constants::load_from_csv_string("TEST1,1\nTEST1,2\n").is_err());

        // Check if we can convert it back and forth
        let data = constants.to_csv()?;
        let constants = Constants::load_from_csv_string(&data)?;

        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        // Duplicate values are reverse-fetched correctly
        let constants: Constants = Constants::load_from_csv_string("TEST1,1\nTEST2,0o1\nTEST3,0x1\nTEST4,2\nTEST5,0x100\n")?;
        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(1i32)), constants.get_by_name("TEST3"));

        let mut names = constants.get_by_value(&Integer::from(1u32)).unwrap().clone();
        names.sort();
        assert_eq!(vec!["TEST1".to_string(), "TEST2".to_string(), "TEST3".to_string()], names);

        Ok(())
    }

    #[test]
    fn test_csv_file() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/constants/test1.csv");

        let constants = Constants::load_from_csv_file(&d)?;

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
        assert_eq!(0, Constants::load_from_csv_string("")?.len());

        Ok(())
    }

    #[test]
    fn test_json() -> SimpleResult<()> {
        let data = "{ \"TEST1\": \"1\", \"TEST3\": \"5\", \"TEST2\": \"100\", \"TEST4\": \"-10000\", \"TEST5\": \"0x100\" }";
        let constants: Constants = Constants::load_from_json_string(data)?;

        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        // Check if we can convert it back and forth
        let data = constants.to_json()?;
        let constants = Constants::load_from_json_string(&data)?;

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
        d.push("testdata/constants/test2.json");

        let constants = Constants::load_from_json_file(&d)?;

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

        let constants: Constants = Constants::load_from_yaml_string(data)?;

        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        let data = constants.to_yaml()?;
        let constants = Constants::load_from_yaml_string(&data)?;

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
        d.push("testdata/constants/test3.yaml");

        let constants = Constants::load_from_yaml_file(&d)?;

        // Do all the same tests as test_yaml()
        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        Ok(())
    }
}
