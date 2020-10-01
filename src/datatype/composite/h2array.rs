use serde::{Serialize, Deserialize};
use crate::datatype::{H2Type, ResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: usize,
    byte_alignment: Option<usize>,
}

impl From<H2Array> for H2Type {
    fn from(o: H2Array) -> H2Type {
        H2Type::from(H2Type::H2Array(o))
    }
}

impl H2Array {
    pub fn new(field_type: H2Type, length: usize) -> Self {
        Self {
            field_type: Box::new(field_type),
            length: length,
            byte_alignment: None,
        }
    }

    pub fn resolve(&self, starting_offset: usize, field_names: Option<Vec<String>>) -> (Vec<ResolvedType>, usize) {
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

    pub fn size(&self) -> usize {
        self.length * self.field_type.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    use crate::datatype::helpers::h2context::{H2Context, NumberDefinition};
    use crate::datatype::basic::h2integer::H2Integer;

    #[test]
    fn test_array() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let context = H2Context::new(&data, 0);

        let i = H2Integer::new(NumberDefinition::U32_BIG);
        let a = H2Array::new(H2Type::from(i), 4);
        let t = H2Type::from(a);

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
