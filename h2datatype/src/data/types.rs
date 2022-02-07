use simple_error::SimpleResult;

use crate::H2Type;
use crate::data::DataTrait;

#[derive(Debug)]
pub struct Types {
    h2type: H2Type,
}

impl Types {
    pub fn get(&self) -> &H2Type {
        &self.h2type
    }
}

impl DataTrait for Types {
    type SerializedType = H2Type;

    /// Load the data from the type that was serialized.
    fn load(data: &Self::SerializedType) -> SimpleResult<Self> {
        Ok(Self {
            h2type: data.clone(),
        })
    }

    /// Get the data in a format that can be serialized
    fn save(&self) -> SimpleResult<Self::SerializedType> {
        Ok(self.h2type.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use crate::Data;

    use generic_number::*;

    #[test]
    fn test_json_file() -> SimpleResult<()> {
        // Load the data
        let constants = Types::load_from_json_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/types/struct.json"].iter().collect::<PathBuf>())?;

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        let resolved = constants.get().resolve(Context::new(&data), None, &Data::default())?;
        assert_eq!(2, resolved.children.len());
        assert_eq!(Integer::from(0x04030201u32), resolved.children.get(0).unwrap().as_integer.unwrap());
        assert_eq!(Integer::from(0xddccbbaau32), resolved.children.get(1).unwrap().as_integer.unwrap());

        Ok(())
    }

    #[test]
    fn test_yaml_file() -> SimpleResult<()> {
        // Load the data
        let constants = Types::load_from_yaml_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/types/struct.yaml"].iter().collect::<PathBuf>())?;

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        let resolved = constants.get().resolve(Context::new(&data), None, &Data::default())?;
        assert_eq!(2, resolved.children.len());
        assert_eq!(Integer::from(0x04030201u32), resolved.children.get(0).unwrap().as_integer.unwrap());
        assert_eq!(Integer::from(0xddccbbaau32), resolved.children.get(1).unwrap().as_integer.unwrap());

        Ok(())
    }
}
