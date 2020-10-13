use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use sized_number::Context;

use crate::datatype::{helpers, H2Type, ResolvedType};
use crate::datatype::basic::H2BasicType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Simple {
    basic_type: Box<H2BasicType>,
    byte_alignment: Option<u64>,
}

impl From<H2Simple> for H2Type {
    fn from(o: H2Simple) -> H2Type {
        H2Type::from(H2Type::H2Simple(o))
    }
}

impl H2Simple {
    pub fn new(basic_type: H2BasicType) -> Self {
        Self {
            basic_type: Box::new(basic_type),
            byte_alignment: None,
        }
    }

    pub fn types_with_offsets(&self, start: u64) -> Vec<(u64, u64, String, H2Type)> {
        let mut result = vec![];

        let end_offset = match self.byte_alignment {
            Some(a) => helpers::round_up(start + self.basic_type.size(), a),
            None    => start + self.basic_type.size(),
        };

        result.push((start, end_offset, "".to_string(), H2Type::from(self.clone())));

        result
    }

    pub fn resolve(&self, starting_offset: u64, field_names: Option<Vec<String>>) -> Vec<ResolvedType> {
        let v: Vec<ResolvedType> = vec![
            ResolvedType {
                offset: starting_offset..(starting_offset + self.basic_type.size()),
                field_names: field_names,
                basic_type: (*self.basic_type).clone(),
            }
        ];

        v
    }

    pub fn size(&self) -> u64 {
        self.basic_type.size()
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

        let t: H2Type = H2Number::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default())).into();
        assert_eq!(4, t.size());

        let resolved = t.resolve();
        assert_eq!(1, resolved.len());
        //assert_eq!(0, resolved[0].offset);
        assert_eq!(None, resolved[0].field_names);

        println!("Type: {:?}", t);
        println!("\nto_string:\n{}", t.to_string(&context)?);

        Ok(())
    }
}
