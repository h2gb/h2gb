use serde::{Serialize, Deserialize};
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

trait H2SimpleTrait {
    fn to_string(&self, data: &Vec<u8>, index: usize) -> String;
    fn length(&self, _data: &Vec<u8>, _index: usize) -> usize;
    fn related(&self, _data: &Vec<u8>, _index: usize) -> Vec<(usize, H2SimpleType)>;
}

trait H2NumericType {
    fn to_number(&self, _data: &Vec<u8>, _index: usize) -> usize;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum NumberFormat {
    Decimal,
    Hex,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Integer {
    number_format: NumberFormat
}

impl H2NumericType for H2Integer {
    fn to_number(&self, data: &Vec<u8>, index: usize) -> usize {
        let mut c = Cursor::new(data);
        c.set_position(index as u64);

        c.read_u32::<BigEndian>().unwrap() as usize
    }
}

impl H2SimpleTrait for H2Integer {
    fn to_string(&self, data: &Vec<u8>, index: usize) -> String {
        let value = self.to_number(data, index);

        match self.number_format {
            NumberFormat::Decimal => {
                format!("{}", value)
            },
            NumberFormat::Hex => {
                format!("{:#010x}", value)
            },
        }
    }

    fn length(&self, _data: &Vec<u8>, _index: usize) -> usize {
        4
    }

    fn related(&self, _data: &Vec<u8>, _index: usize) -> Vec<(usize, H2SimpleType)> {
        vec![]
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Pointer {
    target_type: Box<H2SimpleType>, // TODO: This will be H2Type later
}

impl H2NumericType for H2Pointer {
    fn to_number(&self, data: &Vec<u8>, index: usize) -> usize {
        let mut c = Cursor::new(data);
        c.set_position(index as u64);

        c.read_u32::<BigEndian>().unwrap() as usize
    }
}

impl H2SimpleTrait for H2Pointer {
    fn to_string(&self, data: &Vec<u8>, index: usize) -> String {
        let value = self.to_number(data, index);

        let target_string = self.target_type.to_string(data, value);

        format!("(ref) {:#010x} => {}", value, target_string)
    }

    fn length(&self, _data: &Vec<u8>, _index: usize) -> usize {
        4
    }

    fn related(&self, data: &Vec<u8>, index: usize) -> Vec<(usize, H2SimpleType)> {
        vec![
            (self.to_number(data, index), *self.target_type.clone())
        ]
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2SimpleType {
    Integer(H2Integer),
    Pointer(H2Pointer),
}

impl H2SimpleType {
    fn to_string(&self, data: &Vec<u8>, index: usize) -> String {
        match self {
            Self::Integer(i) => i.to_string(data, index),
            Self::Pointer(p) => p.to_string(data, index),
        }
    }

    fn related(&self, data: &Vec<u8>, index: usize) -> Vec<(usize, H2SimpleType)> {
        match self {
            Self::Integer(i) => i.related(data, index),
            Self::Pointer(p) => p.related(data, index),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Struct {
    // An array of strings and types (which might be other types)
    fields: Vec<(String, H2Type)>,
    byte_alignment: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: usize,
}

trait H2ComplexTrait {
    fn length() -> usize;
    fn related() -> Vec<H2Type>;
    fn to_simple() -> Vec<H2SimpleType>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2ComplexType {
    H2Struct(H2Struct),
    H2Array(H2Array),
}

impl H2ComplexType {
    // Convert a complextype into multiple simpletypes
    fn resolve() -> Vec<H2SimpleType> {
        vec![]
    }

    fn to_simple_types(_data: Vec<u8>, _address: usize) -> Vec<H2SimpleType> {
        vec![]
    }
}

// Complex types should define multiple simple types, eventually
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2Type {
    H2SimpleType(H2SimpleType),
    H2ComplexType(H2ComplexType),
}

impl H2Type {
    fn length() -> usize {
        0
    }

    fn related() -> Vec<H2Type> {
        vec![]
    }

    fn to_simple_types() -> Vec<H2SimpleType> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    //use h2transformer::H2Transformation;

    #[test]
    fn test_datatype() -> SimpleResult<()> {
        let v = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f".to_vec();
        let t = H2SimpleType::Integer(H2Integer {
            number_format: NumberFormat::Hex
        });

        println!("{} => 0x00010203", t.to_string(&v, 0));

        let v = b"\x00\x00\x00\x08AAAABBBBCCCCDDDD".to_vec();
        let t = H2SimpleType::Pointer(H2Pointer {
            target_type: Box::new(H2SimpleType::Integer(H2Integer {
                number_format: NumberFormat::Hex
            }))
        });

        println!("{} => (ref) 0x00000008 (0x42424242)", t.to_string(&v, 0));

        let v = b"\x00\x00\x00\x04\x00\x00\x00\x08BBBBCCCCDDDD".to_vec();
        let t = H2SimpleType::Pointer(H2Pointer {
            target_type: Box::new(H2SimpleType::Pointer(H2Pointer {
                target_type: Box::new(H2SimpleType::Integer(H2Integer {
                    number_format: NumberFormat::Hex
                }))
            }))
        });

        println!("{} => (ref) 0x00000004 ((ref) 0x00000008 (0x42424242))", t.to_string(&v, 0));


        Ok(())
    }
}
