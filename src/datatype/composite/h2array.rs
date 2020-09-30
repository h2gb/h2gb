use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};
use std::iter;

use crate::datatype::H2Type;
use crate::datatype::helpers::h2context::H2Context;
use crate::datatype::simple::H2SimpleType;
use crate::datatype::composite::H2CompositeType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: usize,
    byte_alignment: Option<usize>,
}

impl From<H2Array> for H2Type {
    fn from(o: H2Array) -> H2Type {
        H2Type::from(H2CompositeType::H2Array(o))
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

    pub fn to_simple_types(&self) -> Vec<(Vec<String>, H2SimpleType)> {
        let mut result = Vec::new();

        // TODO: Byte alignment
        for i in 0..self.length {
            for (mut field_name, simple_type) in self.field_type.to_simple_types() {
                field_name.push(i.to_string());
                result.push((
                    field_name,
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
            s.push_str(&format!("Entry {}: ", index[0]));
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
    fn test_array() -> SimpleResult<()> {
        let v = b"AABBCC123456".to_vec();
        let c = H2Context::from((&v, 0));

        let i: H2Type = H2Type::from(
            H2Array::new(H2Type::from(
                H2Integer::new(NumberDefinition::u16_big())
            ), 6)
        );

        println!("Found:\n{}", i.to_string(&c)?);
        println!("\n\nExpected:\n4141\n4242\n4343\n3132\n3334\n3536\n");
        println!("\n\n{}", serde_json::to_string_pretty(&i).unwrap());

        Ok(())
    }

    #[test]
    fn test_pointer_array() -> SimpleResult<()> {
        let v = b"\x04\x08\x0c\x10AAAABBBBCCCCDDDD".to_vec();
        let c = H2Context::from((&v, 0));

        let i: H2Type = H2Type::from(
            H2Array::new(H2Type::from(
                H2Pointer::u8(H2Type::from(
                    H2Integer::new(NumberDefinition::u32_big())
                ))
            ), 4)
        );

        println!("Found:\n{}", i.to_string(&c)?);
        println!("\n\nExpected:\n04 41414141\n08 42424242\n0c 43434343\n10 44444444\n");
        println!("\n\n{}", serde_json::to_string_pretty(&i).unwrap());

        Ok(())
    }
}
