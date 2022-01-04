use std::cmp::Eq;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::str::FromStr;

use bimap::BiMap;
use serde::de::DeserializeOwned;
use simple_error::{SimpleResult, SimpleError, bail};

/// A named collection of constants
pub struct H2Constants<T> {
    constants: BiMap<String, T>,
}

impl<T> H2Constants<T>
where T: Eq + Hash + FromStr + DeserializeOwned
{
    pub fn new_empty() -> Self {
        Self {
            constants: BiMap::new(),
        }
    }

    fn read_csv<R>(reader: R) -> SimpleResult<BiMap<String, T>>
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

            let number: T = record.get(1).ok_or(
                SimpleError::new("Error reading the CSV file")
            )?.parse().map_err(|e| {
                SimpleError::new(format!("Couldn't parse second  CSV field as integer"))
            })?;

            out.insert(name, number);
        }

        Ok(out)
    }

    pub fn load_from_csv_string(data: &str) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_csv(data.as_bytes())?
        })
    }

    pub fn load_from_csv_file(filename: &str) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_csv(io::BufReader::new(File::open(filename).map_err(|e| {
                SimpleError::new(format!("Could not read file: {}", e))
            })?))?
        })
    }

    fn read_yaml<R>(reader: R) -> SimpleResult<BiMap<String, T>>
    where
        R: io::Read
    {
        Ok(serde_yaml::from_reader(reader).map_err(|e| {
            SimpleError::from(e)
        })?)
    }

    pub fn load_from_yaml_string(data: &str) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_yaml(data.as_bytes())?
        })
    }

    pub fn load_from_yaml_file(filename: &str) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_yaml(io::BufReader::new(File::open(filename).map_err(|e| {
                SimpleError::new(format!("Could not read file: {}", e))
            })?))?
        })
    }

    fn read_json<R>(reader: R) -> SimpleResult<BiMap<String, T>>
    where
        R: io::Read
    {
        Ok(serde_json::from_reader(reader).map_err(|e| {
            SimpleError::from(e)
        })?)
    }

    pub fn load_from_json_string(data: &str) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_json(data.as_bytes())?
        })
    }

    pub fn load_from_json_file(filename: &str) -> SimpleResult<Self> {
        Ok(Self {
            constants: Self::read_json(io::BufReader::new(File::open(filename).map_err(|e| {
                SimpleError::new(format!("Could not read file: {}", e))
            })?))?
        })
    }

    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        self.constants.get_by_left(name)
    }

    pub fn get_by_value(&self, value: &T) -> Option<&str> {
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
        let constants: H2Constants<u64> = H2Constants::load_from_csv_string("TEST1,1\nTEST2,100\nTEST3,5\n")?;
        assert_eq!(Some(&1), constants.get_by_name("TEST1"));
        assert_eq!(Some(&100), constants.get_by_name("TEST2"));
        assert_eq!(Some(&5), constants.get_by_name("TEST3"));

        assert_eq!(Some("TEST1"), constants.get_by_value(&1));
        assert_eq!(Some("TEST2"), constants.get_by_value(&100));
        assert_eq!(Some("TEST3"), constants.get_by_value(&5));

        // Negatives fail for unsigned
        assert!(H2Constants::<u64>::load_from_csv_string("TEST1,1\nTEST2,-100\nTEST3,5\n").is_err());

        // Negatives succeed for signed
        assert!(H2Constants::<i64>::load_from_csv_string("TEST1,1\nTEST2,-100\nTEST3,5\n").is_ok());

        // Missing entries fails
        assert!(H2Constants::<u64>::load_from_csv_string("TEST1\nTEST2\nTEST3\n").is_err());

        // Non-numbers fail
        assert!(H2Constants::<u64>::load_from_csv_string("100,TEST1\n").is_err());

        // Blank lines succeedd
        assert_eq!(2, H2Constants::<u64>::load_from_csv_string("TEST1,100\n\n\n\n\nTEST3,200\n")?.len());

        Ok(())
    }

    #[test]
    fn test_csv_file() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_csv_errors() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_csv_empty() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_json() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_json_file() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_json_invalid() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_yaml() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_yaml_file() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_yaml_invalid() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_duplicates() -> SimpleResult<()> {
        Ok(())
    }
}
