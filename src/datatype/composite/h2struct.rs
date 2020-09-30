use serde::{Serialize, Deserialize};

use crate::datatype::{H2Type, ResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Struct {
    // An array of strings and types (which might be other types)
    fields: Vec<(String, H2Type)>,
    byte_alignment: Option<usize>,
}

impl From<H2Struct> for H2Type {
    fn from(o: H2Struct) -> H2Type {
        H2Type::from(H2Type::H2Struct(o))
    }
}

impl H2Struct {
    pub fn new_aligned(fields: Vec<(String, H2Type)>, byte_alignment: usize) -> Self {
        Self {
            fields: fields,
            byte_alignment: Some(byte_alignment),
        }
    }

    pub fn new(fields: Vec<(String, H2Type)>) -> Self {
        Self {
            fields: fields,
            byte_alignment: None,
        }
    }

    pub fn resolve(&self, starting_offset: usize, field_names: Option<Vec<String>>) -> (Vec<ResolvedType>, usize) {
        let mut result: Vec<ResolvedType> = Vec::new();
        let field_names = field_names.unwrap_or(Vec::new());
        let mut offset = starting_offset;

        for (name, field_type) in self.fields.iter() {
            let mut this_field_name = field_names.clone();
            this_field_name.push(name.clone());

            let (mut basic, new_offset) = field_type.resolve_from_offset(Some(offset), Some(this_field_name));
            result.append(&mut basic);

            offset = new_offset;
        }

        (result, offset)
    }

    pub fn size(&self) -> usize {
        self.fields.iter().fold(0, |sum, (_, t)| {
            sum + t.size()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    use crate::datatype::helpers::h2context::{H2Context, NumberDefinition};
    use crate::datatype::basic::h2integer::H2Integer;

    #[test]
    fn test_struct() -> SimpleResult<()> {
        let data = b"\x00\x01\x02\x03\x00\x01\x00\x0f\x0e\x0d\x0c".to_vec();
        let context = H2Context::new(&data, 0);

        let a = H2Struct::new(vec![
            ("field_u32".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u32_big()))),
            ("field_u16".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u16_big()))),
            ("field_u8".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u8()))),
            ("field_u32_little".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u32_little()))),
        ]);
        let t = H2Type::from(a);

        assert_eq!(11, t.size());

        let resolved = t.resolve();
        assert_eq!(4, resolved.len());
        assert_eq!(0, resolved[0].offset);
        assert_eq!(4, resolved[1].offset);
        assert_eq!(6, resolved[2].offset);
        assert_eq!(7, resolved[3].offset);

        println!("Type: {:?}", t);
        println!("\nto_strings:\n{}", t.to_strings(&context)?.join("\n"));

        Ok(())
    }
}
