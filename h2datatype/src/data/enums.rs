use std::collections::HashMap;

use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::Integer;

use crate::data::DataTrait;

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
pub struct Enums {
    by_name: HashMap<String, Integer>,
    by_value: HashMap<Integer, Vec<String>>,

    // Making this "next value" because we need to start at 0
    last_value_added: Option<Integer>,
}

impl Enums {
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

    pub fn get_by_name(&self, name: &str) -> Option<&Integer> {
        self.by_name.get(name)
    }

    pub fn get_by_value(&self, value: &Integer) -> Option<&Vec<String>> {
        self.by_value.get(value)
    }

    pub fn list(&self) -> Vec<(&String, &Integer)> {
        self.by_name.iter().collect()
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }
}

impl DataTrait for Enums {
    type SerializedType = HashMap<String, String>;

    /// Load the data from the type that was serialized.
    fn load(data: &Self::SerializedType) -> SimpleResult<Self> {
        // Convert to String->Integer
        let mut out = Self::new_empty();
        for (name, value) in data {
            // Get the integer
            let value: Integer = value.parse().map_err(|e| SimpleError::new(format!("Couldn't parse integer: {:?}", e)))?;

            // Check for duplicate names
            if out.by_name.contains_key(name) {
                bail!("Duplicate constant value: {}", name);
            }

            // Insert
            out.by_name.insert(name.to_string(), value);

            // Insert or append to the by_value map
            let e = out.by_value.entry(value).or_insert(vec![]);
            e.push(name.to_string());

            // Update the incremental value
            out.last_value_added = Some(value);
        }

        Ok(out)
    }

    fn load_str(data: Vec<(String, Option<Integer>)>) -> SimpleResult<Self> {
        let mut out: HashMap<String, String> = HashMap::new();
        let mut last_value_added: Option<Integer> = None;

        for (name, value) in data {
            // Handle missing values
            let value = match value {
                // If it was set, use it
                Some(v) => v,

                // If it was not set, take the next
                None => {
                    match last_value_added {
                        // If we had a last value, increment it
                        Some(i) => match i.increment() {
                            Some(i) => i,
                            None => bail!("Overflow"),
                        },

                        // If we did not, use 0
                        None => Integer::from(0u32),
                    }
                }
            };

            if out.contains_key(&name) {
                bail!("Duplicate key: {}", name);
            }

            // Save the value
            last_value_added = Some(value);

            out.insert(name, value.to_string());
        }

        Self::load(&out)
    }

    /// Get the data in a format that can be serialized
    fn save(&self) -> SimpleResult<HashMap<String, String>> {
        // Convert to String->String
        let mut h: HashMap<String, String> = HashMap::new();

        for (k, v) in &self.by_name {
            h.insert(k.clone(), v.to_string());
        }

        Ok(h)
    }

    fn save_str(&self) -> SimpleResult<Vec<(String, Integer)>> {
        let mut out: Vec<(String, Integer)> = vec![];

        for (name, value) in &self.by_name {
            out.push((name.clone(), *value))
        }

        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_csv() -> SimpleResult<()> {
        // Most stuff works
        let enums: Enums = Enums::load_from_csv_string("TEST1,1\nTEST2,100\nTEST3,5\nTEST4,-10000\nTEST5,0x100\n")?;
        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        // Missing entries work
        assert!(Enums::load_from_csv_string("TEST1,1\nTEST2\nTEST3,10\n").is_ok());

        // Non-numbers fail
        assert!(Enums::load_from_csv_string("100,TEST1\n").is_err());

        // Blank lines are ignored
        assert_eq!(2, Enums::load_from_csv_string("TEST1,100\n\n\n\n\nTEST3,200\n")?.len());

        // Duplicate names fail
        assert!(Enums::load_from_csv_string("TEST1,1\nTEST1,2\n").is_err());

        // Check if we can convert it back and forth
        let data = enums.to_csv()?;
        let enums = Enums::load_from_csv_string(&data)?;

        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        // Duplicate values are reverse-fetched correctly
        let enums: Enums = Enums::load_from_csv_string("TEST1,1\nTEST2,0o1\nTEST3,0x1\nTEST4,2\nTEST5,0x100\n")?;
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

        let enums = Enums::load_from_csv_file(&d)?;

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
        assert_eq!(0, Enums::load_from_csv_string("")?.len());

        Ok(())
    }

    #[test]
    fn test_json() -> SimpleResult<()> {
        let data = "{ \"TEST1\": \"1\", \"TEST3\": \"5\", \"TEST2\": \"100\", \"TEST4\": \"-10000\", \"TEST5\": \"0x100\" }";
        let enums: Enums = Enums::load_from_json_string(data)?;

        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        // Check if we can convert it back and forth
        let data = enums.to_json()?;
        let enums = Enums::load_from_json_string(&data)?;

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

        let enums = Enums::load_from_json_file(&d)?;

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

        let enums: Enums = Enums::load_from_yaml_string(data)?;

        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), enums.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), enums.get_by_name("TEST5"));

        let data = enums.to_yaml()?;
        let enums = Enums::load_from_yaml_string(&data)?;

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

        let enums = Enums::load_from_yaml_file(&d)?;

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
        let enums: Enums = Enums::load_from_csv_string("TEST1\nTEST2\nTEST3\n")?;
        assert_eq!(Some(&Integer::from(0u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(1u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(2u32)), enums.get_by_name("TEST3"));

        // Jumping ahead works
        let enums: Enums = Enums::load_from_csv_string("TEST1\nTEST2,100\nTEST3\n")?;
        assert_eq!(Some(&Integer::from(0u32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(101u32)), enums.get_by_name("TEST3"));

        // Negatives too
        let enums: Enums = Enums::load_from_csv_string("TEST1,-100\nTEST2\nTEST3,-1\nTEST4\n")?;
        assert_eq!(Some(&Integer::from(-100i32)), enums.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from( -99i32)), enums.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(  -1i32)), enums.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(   0u32)), enums.get_by_name("TEST4"));

        Ok(())
    }
}
