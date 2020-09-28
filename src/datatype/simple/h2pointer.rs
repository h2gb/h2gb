use serde::{Serialize, Deserialize};
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use simple_error::SimpleResult;

use crate::datatype::H2Type;
use crate::datatype::simple::H2SimpleType;
use crate::datatype::helpers::h2context::H2Context;
// use crate::datatype::helpers::number::NumberDefinition;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Pointer {
    target_type: Box<H2Type>,
}

impl From<H2Pointer> for H2Type {
    fn from(o: H2Pointer) -> H2Type {
        H2Type::from(H2SimpleType::Pointer(o))
    }
}

impl H2Pointer {
    pub fn new(target_type: H2Type) -> Self {
        H2Pointer {
            target_type: Box::new(target_type),
        }
    }

    pub fn to_number(&self, context: &H2Context) -> usize {
        let mut c = Cursor::new(context.data);
        c.set_position(context.index as u64);

        c.read_u32::<BigEndian>().unwrap() as usize
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        let value = self.to_number(context);

        let target_context = H2Context {
            data: context.data,
            index: value,
        };

        let target_string = self.target_type.to_string(&target_context)?;

        Ok(format!("(ref) {:#010x} => {}", value, target_string))
    }

    pub fn length(&self) -> usize {
        4
    }

    pub fn related(&self, context: &H2Context) -> Vec<(usize, H2Type)> {
        vec![
            (self.to_number(context), *self.target_type.clone())
        ]
    }

}
