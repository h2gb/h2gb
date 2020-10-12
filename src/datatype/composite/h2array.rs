use serde::{Serialize, Deserialize};
use crate::datatype::{H2Type, ResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: u64,
    byte_alignment: Option<u64>,
}

impl From<H2Array> for H2Type {
    fn from(o: H2Array) -> H2Type {
        H2Type::from(H2Type::H2Array(o))
    }
}

impl H2Array {
    pub fn new(length: u64, field_type: H2Type) -> Self {
        Self {
            field_type: Box::new(field_type),
            length: length,
            byte_alignment: None,
        }
    }

    pub fn resolve(&self, starting_offset: u64, field_names: Option<Vec<String>>) -> (Vec<ResolvedType>, u64) {
        let mut result: Vec<ResolvedType> = Vec::new();
        let field_names = field_names.unwrap_or(Vec::new());
        let mut offset = starting_offset;

        for i in 0..self.length {
            let mut this_field_name = field_names.clone();
            this_field_name.push(i.to_string());

            let (mut basic, new_offset) = self.field_type.resolve_from_offset(Some(offset), Some(this_field_name));
            result.append(&mut basic);

            offset = new_offset;
        }

        (result, offset)
    }

    pub fn size(&self) -> u64 {
        self.length * self.field_type.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};

    use crate::datatype::basic::h2number::H2Number;

    #[test]
    fn test_array() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let context = Context::new(&data);

        let t: H2Type = H2Array::new(4,
            H2Number::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default())).into()
        ).into();

        assert_eq!(16, t.size());

        let resolved = t.resolve();
        assert_eq!(4, resolved.len());
        assert_eq!(0, resolved[0].offset);
        assert_eq!(4, resolved[1].offset);
        assert_eq!(8, resolved[2].offset);
        assert_eq!(12, resolved[3].offset);

        println!("Type: {:?}", t);
        println!("\nto_strings:\n{}", t.to_strings(&context)?.join("\n"));

        Ok(())
    }
}
