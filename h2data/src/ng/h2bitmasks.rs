use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::Integer;

/// A bitmask - ie, a list of binary flags (0/1).
///

#[derive(Debug)]
pub struct H2Bitmasks {
    by_name: HashMap<String, u8>,
    by_position: HashMap<u8, String>,
}

impl H2Bitmasks {
    fn new_empty() -> Self {
        Self {
            by_name: HashMap::new(),
            by_position: HashMap::new(),
        }
    }

    fn add_entry(&mut self, name: &str, position: Integer) -> SimpleResult<()> {
        // Check duplicates
        if self.by_name.contains_key(name) {
            bail!("Duplicate bitmask name: {} -> {}", name, position);
        }

        // Convert it to a u8 since we don't need the full Integer
        let position = position.as_u128();
        if position > 127 {
            bail!("Value {} out of bitmask range");
        }

        // Convert down to a u8 now that we know it's possible
        let position = position as u8;

        // Check duplicates
        if self.by_position.contains_key(&position) {
            bail!("Duplicate bitmask position: {} -> {}", name, position);
        }

        // Insert and prevent duplicates
        self.by_name.insert(name.to_string(), position);
        self.by_position.insert(position, name.to_string());

        Ok(())
    }

    fn load_csv<R>(reader: R) -> SimpleResult<Self>
    where R: io::Read
    {
        let mut out = Self::new_empty();

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(reader);

        for result in rdr.records() {
            let record = result.map_err(|e| {
                SimpleError::new(format!("Couldn't read CSV: {}", e))
            })?;

            // Ensure that there are only two entries per line
            if record.len() > 2 {
                bail!("CSV must be 1 or 2 records per line, this line had {} records", record.len());
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

    pub fn get_by_name(&self, name: &str) -> Option<Integer> {
        self.by_name.get(name).map(|i| Integer::from(*i))
    }

    pub fn get_by_value(&self, value: &Integer) -> Vec<String> {
        let mut value = value.as_u128();
        let mut out = vec![];

        for bit in 0..128 {
            // Check the right-most bit
            if value & (1 << bit) != 0 {
                out.push(match self.by_position.get(&bit) {
                    Some(s) => s.to_string(),
                    None    => format!("Unknown_0x{:x}", 1 << bit),
                });
            }

            // Turn off the bit and check if we're done
            // (this is just for a bit of efficiency)
            value = value & !(1 << bit);
            if value == 0 {
                break;
            }
        }

        out
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
        let bitmasks: H2Bitmasks = H2Bitmasks::load_from_csv_string("TEST1,0\nTEST2,2\nTEST3,5\nTEST4,100\n")?;

        // Test the simple way
        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        // Test the more complicated way
        let flags = bitmasks.get_by_value(&Integer::from(1u32));
        assert_eq!(vec!["TEST1".to_string()], flags);

        // Test 0101 => 5
        let flags = bitmasks.get_by_value(&Integer::from(5u32));
        assert_eq!(vec!["TEST1".to_string(), "TEST2".to_string()], flags);

        // Test 0111 => 7
        let flags = bitmasks.get_by_value(&Integer::from(7u32));
        assert_eq!(vec!["TEST1".to_string(), "Unknown_0x2".to_string(), "TEST2".to_string()], flags);

        // Missing entries fail
        assert!(H2Bitmasks::load_from_csv_string("TEST1,1\nTEST2\nTEST3,10\n").is_err());

        // Non-numbers fail
        assert!(H2Bitmasks::load_from_csv_string("100,TEST1\n").is_err());

        // Values in a bad range fail
        assert!(H2Bitmasks::load_from_csv_string("TEST1,129\n").is_err());

        // Blank lines are ignored
        assert_eq!(2, H2Bitmasks::load_from_csv_string("TEST1,5\n\n\n\n\nTEST3,50\n")?.len());

        // Duplicate names fail
        assert!(H2Bitmasks::load_from_csv_string("TEST1,1\nTEST1,2\n").is_err());

        // Check if we can convert it back and forth
        let data = bitmasks.to_csv()?;
        let bitmasks = H2Bitmasks::load_from_csv_string(&data)?;

        // Test the simple way
        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        // Test the more complicated way
        let flags = bitmasks.get_by_value(&Integer::from(1u32));
        assert_eq!(vec!["TEST1".to_string()], flags);

        // Test 0101 => 5
        let flags = bitmasks.get_by_value(&Integer::from(5u32));
        assert_eq!(vec!["TEST1".to_string(), "TEST2".to_string()], flags);

        // Test 0111 => 7
        let flags = bitmasks.get_by_value(&Integer::from(7u32));
        assert_eq!(vec!["TEST1".to_string(), "Unknown_0x2".to_string(), "TEST2".to_string()], flags);

        Ok(())
    }

    // #[test]
    // fn test_csv_file() -> SimpleResult<()> {
    //     // Load the data
    //     let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //     d.push("testdata/constants/test1.csv");

    //     let constants = H2Bitmasks::load_from_csv_file(&d)?;

    //     // Do all the same tests as test_csv()
    //     assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
    //     assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
    //     assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

    //     Ok(())
    // }

    // #[test]
    // fn test_csv_empty() -> SimpleResult<()> {
    //     assert_eq!(0, H2Bitmasks::load_from_csv_string("")?.len());

    //     Ok(())
    // }

    // #[test]
    // fn test_json() -> SimpleResult<()> {
    //     let data = "{ \"TEST1\": \"1\", \"TEST3\": \"5\", \"TEST2\": \"100\", \"TEST4\": \"-10000\", \"TEST5\": \"0x100\" }";
    //     let constants: H2Bitmasks = H2Bitmasks::load_from_json_string(data)?;

    //     assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
    //     assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
    //     assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

    //     // Check if we can convert it back and forth
    //     let data = constants.to_json()?;
    //     let constants = H2Bitmasks::load_from_json_string(&data)?;

    //     assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
    //     assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
    //     assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

    //     Ok(())
    // }

    // #[test]
    // fn test_json_file() -> SimpleResult<()> {
    //     // Load the data
    //     let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //     d.push("testdata/constants/test2.json");

    //     let constants = H2Bitmasks::load_from_json_file(&d)?;

    //     // Do all the same tests as test_json()
    //     assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
    //     assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
    //     assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

    //     Ok(())
    // }

    // #[test]
    // fn test_yaml() -> SimpleResult<()> {
    //     let data = "---
// TEST4: -10000
// TEST1: 1
// TEST2: 100
// TEST3: 5
// TEST5: 256";

    //     let constants: H2Bitmasks = H2Bitmasks::load_from_yaml_string(data)?;

    //     assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
    //     assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
    //     assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

    //     let data = constants.to_yaml()?;
    //     let constants = H2Bitmasks::load_from_yaml_string(&data)?;

    //     assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
    //     assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
    //     assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

    //     Ok(())
    // }

    // #[test]
    // fn test_yaml_file() -> SimpleResult<()> {
    //     // Load the data
    //     let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    //     d.push("testdata/constants/test3.yaml");

    //     let constants = H2Bitmasks::load_from_yaml_file(&d)?;

    //     // Do all the same tests as test_yaml()
    //     assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
    //     assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
    //     assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

    //     Ok(())
    // }

    // #[test]
    // fn test_autonumber() -> SimpleResult<()> {
    //     // Most stuff works
    //     let constants: H2Bitmasks = H2Bitmasks::load_from_csv_string("TEST1\nTEST2\nTEST3\n")?;
    //     assert_eq!(Some(&Integer::from(0u32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(2u32)), constants.get_by_name("TEST3"));

    //     // Jumping ahead works
    //     let constants: H2Bitmasks = H2Bitmasks::load_from_csv_string("TEST1\nTEST2,100\nTEST3\n")?;
    //     assert_eq!(Some(&Integer::from(0u32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(101u32)), constants.get_by_name("TEST3"));

    //     // Negatives too
    //     let constants: H2Bitmasks = H2Bitmasks::load_from_csv_string("TEST1,-100\nTEST2\nTEST3,-1\nTEST4\n")?;
    //     assert_eq!(Some(&Integer::from(-100i32)), constants.get_by_name("TEST1"));
    //     assert_eq!(Some(&Integer::from( -99i32)), constants.get_by_name("TEST2"));
    //     assert_eq!(Some(&Integer::from(  -1i32)), constants.get_by_name("TEST3"));
    //     assert_eq!(Some(&Integer::from(   0u32)), constants.get_by_name("TEST4"));

    //     Ok(())
    // }
}
