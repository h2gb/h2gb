use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;

use crate::{H2TypeTrait, Data};
use crate::simple::*;
use crate::simple::network::*;
use crate::simple::numeric::*;
use crate::simple::string::*;
use crate::composite::*;

/// The core of this crate - defines any type of value abstractly.
///
/// In general, when consuming this crate, you probably won't be creating an
/// `H2Type` directly; rather, create one of the [`crate::simple`] or
/// [`crate::composite`] types, then use `.into()` to get H2Type.
///
/// Please note that many of the functions here are very expensive, because
/// they have to read the object and iterate every time they're called. If you
/// call `resolve()`, a static version will be created with the fields pre-
/// calculated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum H2Type {
    // Simple
    //H2Pointer(H2Pointer),
    Rgb(Rgb),
    H2Bitmask(H2Bitmask),
    H2Enum(H2Enum),
    H2UUID(H2UUID),
    H2Blob(H2Blob),

    // Numeric
    H2Character(H2Character),
    H2Float(H2Float),
    H2Integer(H2Integer),

    // Network
    IPv4(IPv4),
    IPv6(IPv6),
    MacAddress(MacAddress),
    MacAddress8(MacAddress8),

    // Strings
    H2String(H2String),
    NTString(NTString),
    LPString(LPString),

    // Composite
    H2Array(H2Array),
    H2Struct(H2Struct),

    // "NamedType" - a thin reference to a type defined in &Data
    NamedReference(String),
}

impl H2Type {
    pub fn as_trait<'a>(&'a self, data: &'a Data) -> SimpleResult<&'a dyn H2TypeTrait> {
        Ok(match self {
            // Simple
            H2Type::Rgb(t)       => t,
            H2Type::H2Bitmask(t) => t,
            H2Type::H2Enum(t)    => t,
            H2Type::H2UUID(t)    => t,
            H2Type::H2Blob(t)    => t,

            // Numeric
            H2Type::H2Float(t)     => t,
            H2Type::H2Character(t) => t,
            H2Type::H2Integer(t)   => t,

            // Network
            H2Type::IPv4(t)        => t,
            H2Type::IPv6(t)        => t,
            H2Type::MacAddress(t)  => t,
            H2Type::MacAddress8(t) => t,

            // Complex
            H2Type::H2Array(t)   => t,
            H2Type::H2Struct(t)  => t,

            // Strings
            H2Type::H2String(t)  => t,
            H2Type::NTString(t)  => t,
            H2Type::LPString(t)  => t,

            // External reference
            // TODO: How do we prevent infinite loops?
            H2Type::NamedReference(s) => {
                data.lookup_type(s)?.as_trait(data)?
            },
        })
    }

    pub fn new_named(name: impl AsRef<str>) -> Self {
        Self::NamedReference(name.as_ref().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use crate::Data;

    use generic_number::*;

    #[test]
    fn test_as_trait_named_reference() -> SimpleResult<()> {
        let mut d = Data::new();

        d.insert_type("test_integer", H2Integer::new(
            IntegerReader::U32(Endian::Little),
            DefaultFormatter::new(),
        ))?;

        let t: H2Type = H2Struct::new(vec![
            (
                "field1".to_string(),
                H2Type::new_named("test_integer"),
            ),
            (
                "field2".to_string(),
                H2Type::new_named("test_integer"),
            ),
        ])?.into();

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        let resolved = t.as_trait(&d)?.resolve(Context::new(&data), None, &d)?;

        assert_eq!(2, resolved.children.len());
        assert_eq!(Integer::from(0x04030201u32), resolved.children.get(0).unwrap().as_integer.unwrap());
        assert_eq!(Integer::from(0xddccbbaau32), resolved.children.get(1).unwrap().as_integer.unwrap());

        Ok(())
    }

    #[test]
    fn test_as_trait_failed_reference() -> SimpleResult<()> {
        let mut d = Data::new();

        d.insert_type("test_integer", H2Integer::new(
            IntegerReader::U32(Endian::Little),
            DefaultFormatter::new(),
        ))?;

        let t: H2Type = H2Struct::new(vec![
            (
                "field1".to_string(),
                H2Type::new_named("test_integer"),
            ),
            (
                "field2".to_string(),
                H2Type::new_named("not_test_integer"),
            ),
        ])?.into();

        // We can't equate types, but we know it it's a struct with two U32 LE
        // fields
        let data = b"\x01\x02\x03\x04\xaa\xbb\xcc\xdd".to_vec();
        assert!(t.as_trait(&d)?.resolve(Context::new(&data), None, &d).is_err());

        Ok(())
    }
}
