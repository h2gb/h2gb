use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use sized_number::Context;

use crate::datatype::{helpers, H2StaticType, ResolvedType, PartiallyResolvedType};
use crate::datatype::basic::H2BasicType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Simple {
    basic_type: Box<H2BasicType>,
    byte_alignment: Option<u64>,
}

impl From<H2Simple> for H2StaticType {
    fn from(o: H2Simple) -> H2StaticType {
        H2StaticType::from(H2StaticType::H2Simple(o))
    }
}

impl H2Simple {
    pub fn new(basic_type: H2BasicType) -> Self {
        Self {
            basic_type: Box::new(basic_type),
            byte_alignment: None,
        }
    }

    pub fn new_aligned(byte_alignment: u64, basic_type: H2BasicType) -> Self {
        Self {
            basic_type: Box::new(basic_type),
            byte_alignment: Some(byte_alignment),
        }
    }

    pub fn partially_resolve(&self, start: u64) -> Vec<PartiallyResolvedType> {
        let end_offset = match self.byte_alignment {
            Some(a) => helpers::round_up(start + self.basic_type.size(), a),
            None    => start + self.basic_type.size(),
        };

        vec![PartiallyResolvedType {
            offset: start..end_offset,
            field_name: None,
            field_type: H2StaticType::from(self.clone()),
        }]
    }

    pub fn to_resolved_type(&self, starting_offset: u64, breadcrumbs: Option<Vec<String>>) -> ResolvedType {
        ResolvedType {
            offset: starting_offset..(starting_offset + self.basic_type.size()),
            breadcrumbs: breadcrumbs,
            basic_type: (*self.basic_type).clone(),
        }
    }

    pub fn size(&self) -> u64 {
        match self.byte_alignment {
            Some(a) => helpers::round_up(self.basic_type.size(), a),
            None => self.basic_type.size(),
        }
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        self.basic_type.to_string(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};

    use crate::datatype::basic::h2number::H2Number;

    #[test]
    fn test_simple() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let context = Context::new(&data);

        let t: H2StaticType = H2Number::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default())).into();
        assert_eq!(4, t.size());
        assert_eq!("0x41414141", t.to_string(&context)?);

        let resolved = t.fully_resolve(0, None);
        assert_eq!(1, resolved.len());
        assert_eq!(0..4, resolved[0].offset);
        assert_eq!(None, resolved[0].breadcrumbs);
        assert_eq!("0x41414141", resolved[0].to_string(&context)?);

        Ok(())
    }

    #[test]
    fn test_simple_aligned() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let context = Context::new(&data);

        let t: H2StaticType = H2Simple::new_aligned(4,
            H2BasicType::from(H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())))
        ).into();
        assert_eq!(4, t.size());
        assert_eq!("0x41", t.to_string(&context)?);

        let resolved = t.fully_resolve(0, None);
        assert_eq!(1, resolved.len());
        assert_eq!(0..1, resolved[0].offset);
        assert_eq!(None, resolved[0].breadcrumbs);
        assert_eq!("0x41", resolved[0].to_string(&context)?);

        Ok(())
    }
}
