use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use std::iter;

use crate::datatype::H2Type;
use crate::datatype::composite::H2CompositeType;
use crate::datatype::helpers::h2context::H2Context;
use crate::datatype::simple::H2SimpleType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Struct {
    // An array of strings and types (which might be other types)
    fields: Vec<(String, H2Type)>,
    byte_alignment: Option<usize>,
}

impl From<H2Struct> for H2Type {
    fn from(o: H2Struct) -> H2Type {
        H2Type::from(H2CompositeType::H2Struct(o))
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

    pub fn to_simple_types(&self) -> Vec<(String, H2SimpleType)> {
        let mut result = Vec::new();

        // TODO: Byte alignment
        for (name, field_type) in &self.fields {
            for (field_name, simple_type) in field_type.to_simple_types() {
                result.push((
                    format!("{}.{}", name, field_name),
                    simple_type,
                ));
            }
        }

        result
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        // Clone it so we can change it
        let mut c = context.clone();

        // Get the simple types that represent the array
        let simple_types = self.to_simple_types();

        let mut s = String::from("");
        for (index, t) in simple_types.iter() {
            s.push_str(&format!("Entry {}: ", index));
            s.push_str(&format!("{}\n", t.to_string(&c)?));
            c.increment_index(t.length());
        }

        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    use crate::datatype::simple::h2integer::H2Integer;
    use crate::datatype::simple::h2pointer::H2Pointer;
    use crate::datatype::helpers::h2context::{H2Context, NumberDefinition};

    #[test]
    fn test_struct() -> SimpleResult<()> {
        let v = b"ABCDEF\x00\x01\x02\x03\x00\x01\x02\x03\x04\x05\x06\x07".to_vec();
        let c = H2Context::from((&v, 0));

        let i: H2Type = H2Type::from(
            H2Struct::new(vec![
                ("u8 (0x41)".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u8()))),
                ("u8 (0x42)".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u8()))),
                ("u16 (0x4344)".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u16_big()))),
                ("u16 (0x4645)".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u16_little()))),

                ("u32 (0x00010203)".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u32_big()))),
                ("u64 (0x0001020304050607)".to_string(), H2Type::from(H2Integer::new(NumberDefinition::u64_big()))),
            ]),
        );

        println!("\n{}", i.to_string(&c)?);
        println!("\n\n{}", serde_json::to_string_pretty(&i).unwrap());

        Ok(())
    }

    // #[test]
    // fn test_pointer_array() -> SimpleResult<()> {
    //     let v = b"\x04\x08\x0c\x10AAAABBBBCCCCDDDD".to_vec();
    //     let c = H2Context::from((&v, 0));

    //     let i: H2Type = H2Type::from(
    //         H2Array::new(H2Type::from(
    //             H2Pointer::u8(H2Type::from(
    //                 H2Integer::new(NumberDefinition::u32_big())
    //             ))
    //         ), 4)
    //     );

    //     println!("Found:\n{}", i.to_string(&c)?);
    //     println!("\n\nExpected:\n04 41414141\n08 42424242\n0c 43434343\n10 44444444\n");
    //     println!("\n\n{}", serde_json::to_string_pretty(&i).unwrap());

    //     Ok(())
    // }
}
