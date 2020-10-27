use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};
// use sized_number::{Context, SizedDefinition, SizedDisplay};
use sized_number::Context;

use crate::datatype::{H2Type, PartiallyResolvedType, H2TypeTrait};
// use crate::datatype::basic_type::character::Character;
// use crate::datatype::basic_type::h2number::H2Number;
// use crate::datatype::static_type::h2array::H2Array;
// use crate::datatype::static_type::StaticType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NTString {
}

// impl From<NTString> for DynamicType {
//     fn from(o: NTString) -> DynamicType {
//         DynamicType::from(DynamicType::NTString(o))
//     }
// }

impl NTString {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl H2TypeTrait for NTString {
    fn is_static(&self) -> bool {
        false
    }

    fn static_size(&self) -> SimpleResult<u64> {
        bail!("No size for dynamic types");
    }

    fn name(&self) -> String {
        format!("Null-terminated string")
    }

    // Includes null-terminator
    fn size(&self, context: &Context) -> SimpleResult<u64> {
        let mut size = 0;

        loop {
            let c = context.at(context.position() + size).read_u8()?;

            if c == 0 {
                break;
            }

            if c < 0x20 || c > 0x7E {
                bail!("Not a valid ASCII string");
            }

            size += 1;
        }

        Ok(size + 1)
    }

    fn to_string(&self, context: &Context) -> SimpleResult<String> {
        let size = self.size(context)?;

        if size == 0 {
            Ok("\"\"".to_string())
        } else {
            let v = context.read_bytes((size - 1) as usize)?;
            match std::str::from_utf8(&v) {
                Ok(s)  => Ok(format!("\"{}\"", s)),
                Err(e) => bail!("<invalid string: {}>", e),
            }
        }
    }

    fn children(&self, context: &Context) -> SimpleResult<Vec<PartiallyResolvedType>> {
        let size = self.size(context)?;
        let mut result: Vec<PartiallyResolvedType> = Vec::new();

        // TODO
        // if size > 1 {
        //     result.push(PartiallyResolvedType::new(context.position(), None,
        //         StaticType::from(H2Array::new(
        //             size - 1,
        //             StaticType::from(Character::new()),
        //         )),
        //     ));
        // }

        // result.push(PartiallyResolvedType::new(context.position() + size - 1, None,
        //     StaticType::from(H2Number::new(SizedDefinition::U8, SizedDisplay::Decimal))
        // ));


        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;

    // #[test]
    // fn test_ntstring() -> SimpleResult<()> {
    //     let data = b"\0abcd\0ffff".to_vec();
    //     let context = Context::new(&data);

    //     let s = NTString::new();
    //     assert_eq!(5, s.size(&context.at(1))?);
    //     assert_eq!("\"abcd\"", s.to_string(&context.at(1))?);

    //     let r = s.partially_resolve(&context.at(1))?;
    //     assert_eq!(2, r.len());
    //     assert_eq!("[a, b, c, d]", r[0].to_string(&context.at(1))?);
    //     assert_eq!("0", r[1].to_string(&context.at(1))?);

    //     Ok(())
    // }

    // #[test]
    // fn test_ntstring_no_terminator() -> SimpleResult<()> {
    //     let data = b"\0abcd".to_vec();
    //     let context = Context::new(&data);

    //     let s = NTString::new();
    //     assert!(s.size(&context.at(1)).is_err());
    //     assert!(s.to_string(&context.at(1)).is_err());
    //     assert!(s.partially_resolve(&context.at(1)).is_err());

    //     Ok(())
    // }

    // #[test]
    // fn test_ntstring_zero_length() -> SimpleResult<()> {
    //     let data = b"\0abcd\0ffff".to_vec();
    //     let context = Context::new(&data);

    //     let s = NTString::new();
    //     assert_eq!(1, s.size(&context)?);
    //     assert_eq!("\"\"", s.to_string(&context)?);

    //     let r = s.partially_resolve(&context)?;
    //     assert_eq!(1, r.len());
    //     assert_eq!("0", r[0].to_string(&context)?);

    //     Ok(())
    // }
}
