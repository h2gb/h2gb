use serde::{Serialize, Deserialize};
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use crate::datatype::helpers::{H2Context, NumberFormat};
use crate::datatype::simple::H2SimpleType;
use crate::datatype::H2Type;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Integer {
    number_format: NumberFormat
}

impl From<H2Integer> for H2Type {
    fn from(o: H2Integer) -> H2Type {
        H2Type::from(H2SimpleType::Integer(o))
    }
}

impl H2Integer {
    pub fn new(number_format: NumberFormat) -> Self {
        Self {
            number_format: number_format,
        }
    }

    pub fn to_number(&self, context: &H2Context) -> usize {
        let mut c = Cursor::new(context.data);
        c.set_position(context.index as u64);

        c.read_u32::<BigEndian>().unwrap() as usize
    }

    pub fn to_string(&self, context: &H2Context) -> String {
        let value = self.to_number(context);

        match self.number_format {
            NumberFormat::Decimal => {
                format!("{}", value)
            },
            NumberFormat::Hex => {
                format!("{:#010x}", value)
            },
        }
    }

    pub fn length(&self) -> usize {
        4
    }

    pub fn related(&self, _context: &H2Context) -> Vec<(usize, H2Type)> {
        vec![]
    }
}
