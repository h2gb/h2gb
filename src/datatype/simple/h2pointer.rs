use serde::{Serialize, Deserialize};
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use crate::datatype::H2Type;
use crate::datatype::helpers::H2Context;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Pointer {
    pub target_type: Box<H2Type>, // TODO: This will be H2Type later
}

impl H2Pointer {
    pub fn to_number(&self, context: &H2Context) -> usize {
        let mut c = Cursor::new(context.data);
        c.set_position(context.index as u64);

        c.read_u32::<BigEndian>().unwrap() as usize
    }

    pub fn to_string(&self, context: &H2Context) -> String {
        let value = self.to_number(context);

        let target_context = H2Context {
            data: context.data,
            index: value,
        };

        let target_string = self.target_type.to_string(&target_context);

        format!("(ref) {:#010x} => {}", value, target_string)
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
