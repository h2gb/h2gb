use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::H2Type;
use crate::datatype::basic::H2BasicType;
use crate::datatype::helpers::h2context::{H2Context, Endian, NumberSize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Pointer {
    target_type: Box<H2Type>,
    size: NumberSize,
    endian: Endian,
}

impl From<H2Pointer> for H2Type {
    fn from(o: H2Pointer) -> H2Type {
        H2Type::from(H2BasicType::Pointer(o))
    }
}

impl H2Pointer {
    pub fn new(target_type: H2Type, size: NumberSize, endian: Endian) -> Self {
        H2Pointer {
            target_type: Box::new(target_type),
            size: size,
            endian: endian,
        }
    }

    pub fn name(&self) -> String {
        match self.size {
            NumberSize::Eight     => "ref8",
            NumberSize::Sixteen   => "ref16",
            NumberSize::ThirtyTwo => "ref32",
            NumberSize::SixtyFour => "ref64",
        }.to_string()
    }

    pub fn u8(target_type: H2Type) -> Self {
        Self::new(target_type, NumberSize::Eight, Endian::Big /* Endian doesn't matter for 8-bit */)
    }

    pub fn u16_big(target_type: H2Type) -> Self {
        Self::new(target_type, NumberSize::Sixteen, Endian::Big)
    }

    pub fn u16_little(target_type: H2Type) -> Self {
        Self::new(target_type, NumberSize::Sixteen, Endian::Little)
    }

    pub fn u32_big(target_type: H2Type) -> Self {
        Self::new(target_type, NumberSize::ThirtyTwo, Endian::Big)
    }

    pub fn u32_little(target_type: H2Type) -> Self {
        Self::new(target_type, NumberSize::ThirtyTwo, Endian::Little)
    }

    pub fn u64_big(target_type: H2Type) -> Self {
        Self::new(target_type, NumberSize::ThirtyTwo, Endian::Big)
    }

    pub fn u64_little(target_type: H2Type) -> Self {
        Self::new(target_type, NumberSize::ThirtyTwo, Endian::Little)
    }

    pub fn to_number(&self, context: &H2Context) -> SimpleResult<usize> {
        Ok(context.read_generic(self.endian, self.size)? as usize)
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        let target_context = H2Context {
            data: context.data,
            index: self.to_number(context)?,
        };

        let target_string = self.target_type.to_strings(&target_context)?.join(" / ");

        Ok(format!("(ref) {} => {}", self.size.number_to_hex(self.to_number(&context)? as u64), target_string))
    }

    pub fn size(&self) -> usize {
        self.size.len()
    }

    pub fn related(&self, context: &H2Context) -> SimpleResult<Vec<(usize, H2Type)>> {
        Ok(vec![
            (self.to_number(context)?, *self.target_type.clone())
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    use crate::datatype::helpers::h2context::H2Context;
    use crate::datatype::basic::h2integer::H2Integer;
    use crate::datatype::composite::h2array::H2Array;

    #[test]
    fn test_pointer() -> SimpleResult<()> {
        let data = b"\x00\x08AAAAAA\x00\x01\x02\x03".to_vec();
        let context = H2Context::new(&data, 0);

        let t = H2Type::from(H2Pointer::u16_big(H2Type::from(H2Integer::U32_BIG)));

        assert_eq!(2, t.size());

        println!("Type: {:?}", t);
        println!("\nto_strings:\n{}", t.to_strings(&context)?.join("\n"));

        Ok(())
    }

    #[test]
    fn test_complex_pointer() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x08\x00\x00\x00\x10AABBCCDD\x00\x01\x02\x03\x04\x05\x06\x07\x08".to_vec();
        let context = H2Context::new(&data, 0);

        let t = H2Type::from(H2Array::new(H2Type::from(H2Pointer::u32_big(H2Type::from(H2Array::new(H2Type::from(H2Integer::U16_BIG), 4)))), 2));

        assert_eq!(8, t.size());

        println!("Type: {:?}", t);
        println!("\nto_strings:\n{}", t.to_strings(&context)?.join("\n"));

        Ok(())
    }
}
