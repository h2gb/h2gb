use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

pub mod composite;
pub mod simple;
pub mod helpers;

use helpers::h2context::H2Context;
use simple::H2SimpleType;
use composite::H2CompositeType;

// Composite types should define multiple simple types, eventually
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2Type {
    H2SimpleType(H2SimpleType),
    H2CompositeType(H2CompositeType),
}

impl From<H2SimpleType> for H2Type {
    fn from(o: H2SimpleType) -> H2Type {
        Self::H2SimpleType(o)
    }
}

impl From<H2CompositeType> for H2Type {
    fn from(o: H2CompositeType) -> H2Type {
        Self::H2CompositeType(o)
    }
}

impl H2Type {
    pub fn to_simple_types(&self) -> Vec<H2SimpleType> {
        match self {
            Self::H2SimpleType(t) => vec![t.clone()],
            Self::H2CompositeType(t) => t.to_simple_types(),
        }
    }

    pub fn length(&self) -> usize {
        self.to_simple_types().iter().fold(0, |sum, t| {
            sum + t.length()
        });

        0
    }

    pub fn related(&self, context: &H2Context) -> SimpleResult<Vec<(usize, H2Type)>> {
        let mut result = Vec::new();

        for t in self.to_simple_types() {
            result.append(&mut t.related(context)?);
        };

        Ok(result)
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        Ok(match self {
            Self::H2SimpleType(t) => t.to_string(context)?,
            Self::H2CompositeType(t) => t.to_string(context),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    use simple::h2integer::H2Integer;
    use simple::h2pointer::H2Pointer;
    use helpers::h2context::NumberDefinition;

    #[test]
    fn test_datatype() -> SimpleResult<()> {
        let v = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f".to_vec();
        let i: H2Type = H2Integer::new(NumberDefinition::u32_big()).into();

        println!("{} => 0x00010203", i.to_string(&(&v, 0).into())?);
        println!("{}", serde_json::to_string_pretty(&i).unwrap());
        println!("");

        let v = b"\x00\x00\x00\x08AAAABBBBCCCCDDDD".to_vec();
        let t: H2Type = H2Pointer::new(H2Integer::new(NumberDefinition::u32_big()).into()).into();

        println!("{} => (ref) 0x00000008 (0x42424242)", t.to_string(&(&v, 0).into())?);
        println!("{}", serde_json::to_string_pretty(&t).unwrap());
        println!("");

        let v = b"\x00\x00\x00\x04\x00\x00\x00\x08BBBBCCCCDDDD".to_vec();
        let t: H2Type = H2Pointer::new(H2Pointer::new(H2Integer::new(NumberDefinition::u32_big()).into()).into()).into();
        println!("{} => (ref) 0x00000004 ((ref) 0x00000008 (0x42424242))", t.to_string(&(&v, 0).into())?);
        println!("{}", serde_json::to_string_pretty(&t).unwrap());
        println!("");

        Ok(())
    }
}
