use simple_error::SimpleResult;

use crate::H2Type;
use crate::data::DataTrait;

#[derive(Debug)]
pub struct Types {
    h2type: H2Type,
}

impl Types {
    pub fn new(t: impl Into<H2Type>) -> Self {
        Self {
            h2type: t.into(),
        }
    }

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

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use crate::Data;

    use generic_number::*;
    use crate::composite::*;
    use crate::simple::numeric::*;

    // Note: We're only testing strings, not files, since as of the time of
    // writing, the Type format isn't 100% stable.
    #[test]
    fn test_json_type() -> SimpleResult<()> {
        let t = H2Struct::new(vec![
            (
                "field1".to_string(),
                H2Integer::new(
                    IntegerReader::U32(Endian::Little),
                    DefaultFormatter::new(),
                ).into()
            ),
            (
                "field2".to_string(),
                H2Integer::new(
                    IntegerReader::U32(Endian::Little),
                    DefaultFormatter::new(),
                ).into()
            ),
        ])?;

        let as_string = serde_json::to_string_pretty(&Into::<H2Type>::into(t)).unwrap();
        let constants = Types::load_from_json_string(&as_string).unwrap();

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        let resolved = constants.get().as_trait(&Data::default())?.resolve(Context::new(&data), None, &Data::default())?;
        assert_eq!(2, resolved.children.len());
        assert_eq!(Integer::from(0x04030201u32), resolved.children.get(0).unwrap().as_integer.unwrap());
        assert_eq!(Integer::from(0xddccbbaau32), resolved.children.get(1).unwrap().as_integer.unwrap());

        Ok(())
    }

    #[test]
    fn test_yaml_file() -> SimpleResult<()> {
        let t = H2Struct::new(vec![
            (
                "field1".to_string(),
                H2Integer::new(
                    IntegerReader::U32(Endian::Little),
                    DefaultFormatter::new(),
                ).into()
            ),
            (
                "field2".to_string(),
                H2Integer::new(
                    IntegerReader::U32(Endian::Little),
                    DefaultFormatter::new(),
                ).into()
            ),
        ])?;

        let as_string = serde_yaml::to_string(&Into::<H2Type>::into(t)).unwrap();
        let constants = Types::load_from_yaml_string(&as_string).unwrap();

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        let resolved = constants.get().as_trait(&Data::default())?.resolve(Context::new(&data), None, &Data::default())?;
        assert_eq!(2, resolved.children.len());
        assert_eq!(Integer::from(0x04030201u32), resolved.children.get(0).unwrap().as_integer.unwrap());
        assert_eq!(Integer::from(0xddccbbaau32), resolved.children.get(1).unwrap().as_integer.unwrap());

        Ok(())
    }
}
