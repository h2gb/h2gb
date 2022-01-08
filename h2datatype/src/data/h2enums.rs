use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::collections::HashMap;

use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::Integer;

/// An enumeration - ie, a list of numbered values / options.
///
/// An enum consists of a bunch of names, with "optional" values - that is, the
/// values don't need to be specified in CSV, and if they aren't, sequential
/// values will be assigned. Values DO need to be included in JSON and YAML,
/// since they are technically unordered.
///
/// We automatically generate values the same way that C does - that is, if
/// a value is specified, we use it; if a value is not specified, we use either
/// `0` (if it's first), or one more than the previous value (otherwise). For
/// example, the following code outputs `2, 10, 1, 2`:
///
/// ```c
/// #include <stdio.h>
///
/// typedef enum {
///   A = 2,
///   B = 10,
///   C = 1,
///   D
/// } test_t;
///
/// int main(int argc, char *argv[])
/// {
///   printf("%d %d %d %d\n", A, B, C, D);
///   return 0;
/// }
/// ```
///
/// Obviously, much like constants, enumerations do not require unique values,
/// but do require unique names.
///
/// Enums can be fetched either by name or by value. Fetching by value
/// returns the name associated with it (and that's probably the most
/// common use case)
///
/// These values can be loaded from a variety of file formats, and are parsed
/// into [`generic_number::Integer`] instances. The formats are all, ultimately,
/// key value pairs, where both the key (name of constant) and value (the value
/// of the constant) are strings, which are parsed into the best fitting
/// `Integer`. We also support prefixes (like `0x` for hex, `0o` for octal,
/// etc).
#[derive(Debug)]
pub struct H2Enums {
    by_name: HashMap<String, Integer>,
    by_value: HashMap<Integer, Vec<String>>,

    // Making this "next value" because we need to start at 0
    last_value_added: Option<Integer>,
}

impl H2Enums {
    fn new_empty() -> Self {
        Self {
            by_name: HashMap::new(),
            by_value: HashMap::new(),
            last_value_added: None,
        }
    }

    /// Retrieve the next automatic value, but does NOT update it
    fn autovalue(&self) -> SimpleResult<Integer> {
        // Check the last value
        match self.last_value_added {
            // If it exists, increment it
            Some(i) => match i.increment() {
                Some(i) => Ok(i),
                // Check for overflows
                None => bail!("Overflow"),
            },
            // If there's no previous value, start at 0
            None  => Ok(Integer::from(0u32)),
        }
    }

    fn add_entry(&mut self, name: &str, value: Option<Integer>) -> SimpleResult<()> {
        // Check for duplicate names
        if self.by_name.contains_key(name) {
            bail!("Duplicate constant value: {}", name);
        }

        // Get the value, or the next incremental value
        let value = match value {
            Some(v) => v,
            None    => self.autovalue()?,
        };

        // Insert
        self.by_name.insert(name.to_string(), value);

        // Insert or append to the by_value map
        let e = self.by_value.entry(value).or_insert(vec![]);
        e.push(name.to_string());

        // Update the incremental value
        self.last_value_added = Some(value);

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
            let value: Option<Integer> = match record.get(1) {
                Some(v) => Some(v.parse().map_err(|_| SimpleError::new(format!("Couldn't parse second CSV field as integer")))?),
                None => None,
            };

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
        let h: HashMap<String, Option<String>> = serde_yaml::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't read YAML file as String->String mapping: {:?}", e))
        })?;

        // Convert to String->Integer
        let mut out = Self::new_empty();
        for (name, value) in h.into_iter() {
            // Get the integer
            let value: Option<Integer> = match value {
                Some(v) => Some(v.parse().map_err(|_| SimpleError::new(format!("Couldn't parse second YAML field as integer")))?),
                None => None,
            };

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
        let h: HashMap<String, Option<String>> = serde_json::from_reader(reader).map_err(|e| {
            SimpleError::new(format!("Couldn't read JSON file as String->String mapping: {:?}", e))
        })?;

        // Convert to String->Integer
        let mut out = Self::new_empty();
        for (name, value) in h.into_iter() {
            // Get the integer
            let value: Option<Integer> = match value {
                Some(v) => Some(v.parse().map_err(|_| SimpleError::new(format!("Couldn't parse second JSON field as integer")))?),
                None => None,
            };

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
        let enums: H2Enums = H2Enums::load_from_csv_string("TEST1,1\nTEST2,100\nTEST3,5\nTEST4,-10000\nTEST5,0x100\n")?;
        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        // Missing entries work
        assert!(H2Enums::load_from_csv_string("TEST1,1\nTEST2\nTEST3,10\n").is_ok());

        // Non-numbers fail
        assert!(H2Enums::load_from_csv_string("100,TEST1\n").is_err());

        // Blank lines are ignored
        assert_eq!(2, H2Enums::load_from_csv_string("TEST1,100\n\n\n\n\nTEST3,200\n")?.len());

        // Duplicate names fail
        assert!(H2Enums::load_from_csv_string("TEST1,1\nTEST1,2\n").is_err());

        // Check if we can convert it back and forth
        let data = enums.to_csv()?;
        let enums = H2Enums::load_from_csv_string(&data)?;

        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        // Duplicate values are reverse-fetched correctly
        let enums: H2Enums = H2Enums::load_from_csv_string("TEST1,1\nTEST2,0o1\nTEST3,0x1\nTEST4,2\nTEST5,0x100\n")?;
        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(1i32)), enums.get_by_name("TEST3"));

        let mut names = enums.get_by_value(&Integer::from(1u32)).unwrap().clone();
        names.sort();
        assert_eq!(vec!["TEST1".to_string(), "TEST2".to_string(), "TEST3".to_string()], names);

        Ok(())
    }

    #[test]
    fn test_csv_file() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/enums/test1.csv");

        let enums = H2Enums::load_from_csv_file(&d)?;

        // Do all the same tests as test_csv()
        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(101u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(102i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_csv_empty() -> SimpleResult<()> {
        assert_eq!(0, H2Enums::load_from_csv_string("")?.len());

        Ok(())
    }

    #[test]
    fn test_json() -> SimpleResult<()> {
        let data = "{ \"TEST1\": \"1\", \"TEST3\": \"5\", \"TEST2\": \"100\", \"TEST4\": \"-10000\", \"TEST5\": \"0x100\" }";
        let enums: H2Enums = H2Enums::load_from_json_string(data)?;

        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        // Check if we can convert it back and forth
        let data = enums.to_json()?;
        let enums = H2Enums::load_from_json_string(&data)?;

        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_json_file() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/enums/test2.json");

        let enums = H2Enums::load_from_json_file(&d)?;

        // Do all the same tests as test_json()
        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

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

        let enums: H2Enums = H2Enums::load_from_yaml_string(data)?;

        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        let data = enums.to_yaml()?;
        let enums = H2Enums::load_from_yaml_string(&data)?;

        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_yaml_file() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/enums/test3.yaml");

        let enums = H2Enums::load_from_yaml_file(&d)?;

        // Do all the same tests as test_yaml()
        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_autonumber() -> SimpleResult<()> {
        // Most stuff works
        let enums: H2Enums = H2Enums::load_from_csv_string("TEST1\nTEST2\nTEST3\n")?;
        assert_eq!(Some(&Integer::from(0u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(2u32)), enums.get_by_name("TEST3"));

        // Jumping ahead works
        let enums: H2Enums = H2Enums::load_from_csv_string("TEST1\nTEST2,100\nTEST3\n")?;
        assert_eq!(Some(&Integer::from(0u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(101u32)), enums.get_by_name("TEST3"));

        // Negatives too
        let enums: H2Enums = H2Enums::load_from_csv_string("TEST1,-100\nTEST2\nTEST3,-1\nTEST4\n")?;
        assert_eq!(Some(&Integer::from(-100i32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from( -99i32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(  -1i32)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(   0u32)), enums.get_by_name("TEST4"));

        Ok(())
    }
}
