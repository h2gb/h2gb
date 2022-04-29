use std::collections::HashMap;
use std::str::FromStr;

use simple_error::{SimpleResult, SimpleError, bail};

use generic_number::{Integer, IntegerRenderer};

use crate::data::traits::{DataTrait, Lookupable};

/// Configures how a bitmask is displayed.
pub struct BitmaskOptions {
    /// A prefix, followed by a renderer
    unknown_renderer: Option<(String, IntegerRenderer)>,

    /// Show bits that are turned off (ie, 0 bits)?
    ///
    /// This typically creates much longer output, because more values are
    /// displayed - instead of just `A` you get `A | ~B | ~C | ~D | ...`.
    show_negatives: bool,
}

impl BitmaskOptions {
    pub fn new(unknown_renderer: Option<(String, IntegerRenderer)>, show_negatives: bool) -> Self {
        Self {
            unknown_renderer: unknown_renderer,
            show_negatives: show_negatives,
        }
    }
}

impl Default for BitmaskOptions {
    fn default() -> Self {
        Self {
            unknown_renderer: None,
            show_negatives: false,
        }
    }
}

/// A bitmask - ie, a list of binary flags.
///
/// A bitmask is effectively a list of name->value pairs, much like constants
/// and enums, but in practice it has some differences.
///
/// A bitmask has, at most, 128 entries. Each entry is a number, which
/// corresponds to a bit. So you might have `A => 0`, `B => 1`, `C => 2`, etc.
/// When a value is fetched, it's matched against each bit and every "on" bit's
/// label is returned (unless `BitmaskOptions::show_negatives` is set, then
/// everything is returned).
#[derive(Debug, Default)]
pub struct Bitmasks {
    by_name: HashMap<String, u8>,
    by_position: HashMap<u8, String>,
}

impl Bitmasks {
    fn new() -> Self {
        Self::default()
    }

    pub fn get_by_name(&self, name: impl AsRef<str>) -> Option<Integer> {
        self.by_name.get(name.as_ref()).map(|i| Integer::from(*i))
    }

    pub fn get_by_value(&self, value: impl Into<Integer>, options: &BitmaskOptions) -> Vec<String> {
        let value: Integer = value.into();
        let mut value: u128 = value.into();
        let mut out = vec![];

        for bit in 0..128u8 {
            // Mask out the bit
            if value & (1 << bit) != 0 {
                // Check if we have a definition for it
                match (self.by_position.get(&bit), &options.unknown_renderer) {
                    // If the bitmask exists, use it
                    (Some(s), _) => out.push(s.to_string()),

                    // If it doesn't exist, check if we have a renderer
                    (None, Some((s,r))) => out.push(format!("{}{}", s, r.render_integer(Integer::from(1u32 << bit)))),

                    // If we have no unknown renderer, skip
                    (None, None) => (),
                };
            } else if options.show_negatives {
                match self.by_position.get(&bit) {
                    Some(s) => {
                        out.push(format!("~{}", s.to_string()));
                    },
                    None => (),
                }
            }

            if !options.show_negatives {
                // Turn off the bit and check if we're done
                // (this is just for a bit of efficiency if we aren't displaying negatives)
                value = value & !(1 << bit);
                if value == 0 {
                    break;
                }
            }
        }

        out
    }

    pub fn list(&self) -> Vec<(&String, &u8)> {
        self.by_name.iter().collect()
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }
}

impl DataTrait for Bitmasks {
    type SerializedType = HashMap<String, String>;

    /// Load the data from the type that was serialized.
    fn load(data: &Self::SerializedType) -> SimpleResult<Self> {
        // Convert to String->Integer
        let mut out = Self::new();
        for (name, value) in data {
            // Get the integer
            let position = Integer::from_str(&value).map_err(|e| {
                SimpleError::new(format!("Couldn't parse integer: {:?}", e))
            })?;

            // Check duplicates
            if out.by_name.contains_key(name) {
                bail!("Duplicate bitmask name: {} -> {}", name, position);
            }

            // Convert it to a u8 since we don't need the full Integer
            let position: u128 = position.into();
            if position > 127 {
                bail!("Value {} out of bitmask range", position);
            }

            // Convert down to a u8 now that we know it's possible
            let position = position as u8;

            // Check duplicates
            if out.by_position.contains_key(&position) {
                bail!("Duplicate bitmask position: {} -> {}", name, position);
            }

            // Insert and prevent duplicates
            out.by_name.insert(name.to_string(), position);
            out.by_position.insert(position, name.to_string());
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
            out.push((name.clone(), Integer::from(*value)))
        }

        Ok(out)
    }
}

impl Lookupable for Bitmasks {
    type LookupBy = Integer;
    type LookupResult = Vec<String>;
    type LookupOptions = Option<BitmaskOptions>;

    /// Find a specific value in an enum based on an [`Integer`].
    ///
    /// Empty list means no value was found, an `Err` is returned if the name does
    /// not exist.
    fn lookup_options(&self, value: impl Into<Integer>, options: Option<BitmaskOptions>) -> Vec<String> {
        match options {
            Some(o) => self.get_by_value(value, &o),
            None => self.get_by_value(value, &BitmaskOptions::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use simple_error::SimpleResult;
    use generic_number::HexFormatter;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_csv() -> SimpleResult<()> {
        // Most stuff works
        let bitmasks: Bitmasks = Bitmasks::load_from_csv_string("TEST1,0\nTEST2,2\nTEST3,5\nTEST4,100\n")?;

        // Test the simple way
        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        // Test the more complicated way
        let flags = bitmasks.get_by_value(1, &BitmaskOptions::new(None, false));
        assert_eq!(vec!["TEST1".to_string()], flags);

        // Test 0101 => 5
        let flags = bitmasks.get_by_value(5, &BitmaskOptions::new(None, false));
        assert_eq!(vec!["TEST1".to_string(), "TEST2".to_string()], flags);

        // Test 0111 => 7 - no unknown_renderer set
        let flags = bitmasks.get_by_value(7, &BitmaskOptions::new(None, false));
        assert_eq!(vec!["TEST1".to_string(), "TEST2".to_string()], flags);

        // Test 0111 => 7 - unknown_renderer set
        let renderer: (String, IntegerRenderer) = ("Unknown_".to_string(), HexFormatter::new(false, true, false).into());
        let flags = bitmasks.get_by_value(7, &BitmaskOptions::new(Some(renderer), false));
        assert_eq!(vec!["TEST1".to_string(), "Unknown_0x2".to_string(), "TEST2".to_string()], flags);

        // Missing entries fail
        assert!(Bitmasks::load_from_csv_string("TEST1,1\nTEST2\nTEST3,10\n").is_err());

        // Non-numbers fail
        assert!(Bitmasks::load_from_csv_string("100,TEST1\n").is_err());

        // Values in a bad range fail
        assert!(Bitmasks::load_from_csv_string("TEST1,129\n").is_err());

        // Blank lines are ignored
        assert_eq!(2, Bitmasks::load_from_csv_string("TEST1,5\n\n\n\n\nTEST3,50\n")?.len());

        // Duplicate names fail
        assert!(Bitmasks::load_from_csv_string("TEST1,1\nTEST1,2\n").is_err());

        // Check if we can convert it back and forth
        let data = bitmasks.to_csv()?;
        let bitmasks = Bitmasks::load_from_csv_string(&data)?;

        // Test the simple way
        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_csv_file() -> SimpleResult<()> {
        // Load the data
        let bitmasks = Bitmasks::load_from_csv_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/bitmasks/test1.csv"].iter().collect::<PathBuf>())?;

        // Do all the same tests as test_csv()
        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_csv_empty() -> SimpleResult<()> {
        assert_eq!(0, Bitmasks::load_from_csv_string("")?.len());

        Ok(())
    }

    #[test]
    fn test_json() -> SimpleResult<()> {
        // Most stuff works
        let data = "{ \"TEST1\": \"0\", \"TEST3\": \"5\", \"TEST2\": \"2\", \"TEST4\": \"100\" }";
        let bitmasks: Bitmasks = Bitmasks::load_from_json_string(data)?;

        // Test the simple way
        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        // Check if we can convert it back and forth
        let data = bitmasks.to_json()?;
        let bitmasks = Bitmasks::load_from_json_string(&data)?;

        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_json_file() -> SimpleResult<()> {
        // Load the data
        let bitmasks = Bitmasks::load_from_json_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/bitmasks/test2.json"].iter().collect::<PathBuf>())?;

        // Do all the same tests as test_json()
        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_yaml() -> SimpleResult<()> {
        let data = "---
TEST4: 0x64
TEST1: 0
TEST2: 2
TEST3: 5";

        let bitmasks: Bitmasks = Bitmasks::load_from_yaml_string(data)?;

        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        let data = bitmasks.to_yaml()?;
        let bitmasks = Bitmasks::load_from_yaml_string(&data)?;

        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_yaml_file() -> SimpleResult<()> {
        // Load the data
        let bitmasks = Bitmasks::load_from_yaml_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/bitmasks/test3.yaml"].iter().collect::<PathBuf>())?;

        // Do all the same tests as test_yaml()
        assert_eq!(Some(Integer::from(0u32)), bitmasks.get_by_name("TEST1"));
        assert_eq!(Some(Integer::from(2u32)), bitmasks.get_by_name("TEST2"));
        assert_eq!(Some(Integer::from(5u32)), bitmasks.get_by_name("TEST3"));
        assert_eq!(Some(Integer::from(100u32)), bitmasks.get_by_name("TEST4"));
        assert_eq!(None, bitmasks.get_by_name("TEST5"));

        Ok(())
    }

    #[test]
    fn test_show_negative() -> SimpleResult<()> {
        // Most stuff works
        let bitmasks: Bitmasks = Bitmasks::load_from_csv_string("TEST0,0\nTEST1,1\nTEST2,2\nTEST3,3\n")?;

        // Test the simple way
        let mut out = bitmasks.get_by_value(7, &BitmaskOptions::new(None, false));
        out.sort();
        assert_eq!(vec!["TEST0", "TEST1", "TEST2"], out);

        let mut out = bitmasks.get_by_value(7, &BitmaskOptions::new(None, true));
        out.sort();
        assert_eq!(vec!["TEST0", "TEST1", "TEST2", "~TEST3"], out);

        Ok(())
    }
}
