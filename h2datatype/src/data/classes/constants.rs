use std::str::FromStr;
use std::collections::HashMap;

use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::Integer;

use crate::data::traits::{DataTrait, Lookupable};

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

    pub fn get_by_name(&self, name: &str) -> Option<&Integer> {
        self.by_name.get(name)
    }

    pub fn get_by_value(&self, value: &Integer) -> Vec<String> {
        match self.by_value.get(value) {
            Some(v) => v.to_owned(),
            None => Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn list(&self) -> Vec<(&String, &Integer)> {
        self.by_name.iter().collect()
    }
}

impl DataTrait for Constants {
    type SerializedType = HashMap<String, String>;

    /// Load the data from the type that was serialized.
    fn load(data: &HashMap<String, String>) -> SimpleResult<Self> {
        // Convert the data to String->Integer
        let mut out = Self::new_empty();
        for (name, value) in data {
            // Get the integer
            let value = Integer::from_str(&value).map_err(|e| {
                SimpleError::new(format!("Couldn't parse integer: {:?}", e))
            })?;

            // Check for duplicate names (I don't think this can actually happen)
            if out.by_name.contains_key(name) {
                bail!("Duplicate constant value: {}", name);
            }

            // Insert
            out.by_name.insert(name.to_string(), value);

            // Insert or append to the by_value map
            let e = out.by_value.entry(value).or_insert(vec![]);
            e.push(name.to_string());
        }

        Ok(out)
    }

    fn load_str(data: Vec<(String, Option<Integer>)>) -> SimpleResult<Self> {
        let mut out: HashMap<String, String> = HashMap::new();

        for (name, value) in data {
            let value = match value {
                Some(v) => v,
                None => bail!("Constant is missing a value: {}", name),
            };

            if out.contains_key(&name) {
                bail!("Duplicate key: {}", name);
            }

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

impl Lookupable for Constants {
    type LookupBy = Integer;
    type LookupResult = Vec<String>;
    type LookupOptions = ();

    /// Find a specific value in an enum based on an [`Integer`].
    ///
    /// Empty list means no value was found, an `Err` is returned if the name does
    /// not exist.
    fn lookup(&self, value: &Integer, options: ()) -> Vec<String> {
        self.get_by_value(value)
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

        let mut names = constants.get_by_value(&Integer::from(1u32));
        names.sort();
        assert_eq!(vec!["TEST1".to_string(), "TEST2".to_string(), "TEST3".to_string()], names);

        Ok(())
    }

    #[test]
    fn test_csv_file() -> SimpleResult<()> {
        // Load the data
        let constants = Constants::load_from_csv_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test1.csv"].iter().collect::<PathBuf>())?;

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
        let constants = Constants::load_from_json_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test2.json"].iter().collect::<PathBuf>())?;

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
        let constants = Constants::load_from_yaml_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/constants/test3.yaml"].iter().collect::<PathBuf>())?;

        // Do all the same tests as test_yaml()
        assert_eq!(Some(&Integer::from(1u32)), constants.get_by_name("TEST1"));
        assert_eq!(Some(&Integer::from(100u32)), constants.get_by_name("TEST2"));
        assert_eq!(Some(&Integer::from(5u8)), constants.get_by_name("TEST3"));
        assert_eq!(Some(&Integer::from(-10000i32)), constants.get_by_name("TEST4"));
        assert_eq!(Some(&Integer::from(0x100u32)), constants.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_ron() -> SimpleResult<()> {
        let data = "{ \"TEST1\": \"1\", \"TEST3\": \"5\", \"TEST2\": \"100\", \"TEST4\": \"-10000\", \"TEST5\": \"0x100\" }";
        let constants: Constants = Constants::load_from_ron_string(data)?;

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

}
