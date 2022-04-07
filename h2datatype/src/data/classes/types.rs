use simple_error::SimpleResult;

use crate::H2Type;
use crate::data::traits::DataTrait;

/// A Types entry is literally just an alias of [`H2Type`].
pub type Types = H2Type;

impl DataTrait for H2Type {
    type SerializedType = Self;

    /// Load the data from the type that was serialized.
    fn load(data: &Self::SerializedType) -> SimpleResult<Self> {
        Ok(data.to_owned())
    }

    /// Get the data in a format that can be serialized
    fn save(&self) -> SimpleResult<Self::SerializedType> {
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use crate::Data;
    use crate::data::DataTrait;
    // use crate::composite::*;
    // use crate::simple::numeric::*;

    use generic_number::*;

    #[test]
    fn test_json_file() -> SimpleResult<()> {
        // In case I need to re-generate:
        // let s: H2Type = H2Struct::new(vec![
        //     ("a".to_string(),    H2Integer::new(IntegerReader::U32(Endian::Little), DefaultFormatter::new()).into()),
        //     ("b".to_string(),    H2Integer::new(IntegerReader::U32(Endian::Little), DefaultFormatter::new()).into()),
        // ]).unwrap().into();
        //  println!("{}", serde_json::to_string_pretty(&s).unwrap());

        let constants = Types::load_from_json_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/types/struct.json"].iter().collect::<PathBuf>())?;

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        let resolved = constants.as_trait(&Data::default())?.resolve(Context::new(&data), None, &Data::default())?;
        assert_eq!(2, resolved.children.len());
        assert_eq!(Integer::from(0x04030201u32), resolved.children.get(0).unwrap().as_integer.unwrap());
        assert_eq!(Integer::from(0xddccbbaau32), resolved.children.get(1).unwrap().as_integer.unwrap());

        Ok(())
    }

    #[test]
    fn test_yaml_file() -> SimpleResult<()> {
        // In case I need to re-generate:
        // let s: H2Type = H2Struct::new(vec![
        //     ("a".to_string(),    H2Integer::new(IntegerReader::U32(Endian::Little), DefaultFormatter::new()).into()),
        //     ("b".to_string(),    H2Integer::new(IntegerReader::U32(Endian::Little), DefaultFormatter::new()).into()),
        // ]).unwrap().into();
        // println!("{}", serde_yaml::to_string(&s).unwrap());

        // Load the data
        let constants = Types::load_from_yaml_file(&[env!("CARGO_MANIFEST_DIR"), "testdata/types/struct.yaml"].iter().collect::<PathBuf>())?;

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        let resolved = constants.as_trait(&Data::default())?.resolve(Context::new(&data), None, &Data::default())?;
        assert_eq!(2, resolved.children.len());
        assert_eq!(Integer::from(0x04030201u32), resolved.children.get(0).unwrap().as_integer.unwrap());
        assert_eq!(Integer::from(0xddccbbaau32), resolved.children.get(1).unwrap().as_integer.unwrap());

        Ok(())
    }
}
