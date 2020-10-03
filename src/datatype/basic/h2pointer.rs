use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::H2Type;
use crate::datatype::basic::H2BasicType;
use crate::datatype::helpers::sized_number::{SizedNumber, NumberFormat, SizedDisplay};
use crate::datatype::helpers::H2Context;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Pointer {
    number_format: NumberFormat,
    display_format: SizedDisplay,
    target_type: Box<H2Type>,
}

impl From<H2Pointer> for H2Type {
    fn from(o: H2Pointer) -> H2Type {
        H2Type::from(H2BasicType::Pointer(o))
    }
}

impl H2Pointer {
    pub fn new(number_format: NumberFormat, display_format: SizedDisplay, target_type: H2Type) -> Self {
        H2Pointer {
            target_type: Box::new(target_type),
            number_format: number_format,
            display_format: display_format,
        }
    }

    pub fn to_number(&self, context: &H2Context) -> SimpleResult<SizedNumber> {
        self.number_format.read(context)
    }

    pub fn to_string(&self, context: &H2Context) -> SimpleResult<String> {
        // Read the current value
        let number = self.number_format.read(context)?;
        let pointer_display = self.display_format.to_string(number);

        // Read the target from a separate context
        let mut target_context = context.clone();
        target_context.set_position(number.to_index()?);
        let target_display = match self.target_type.to_strings(&target_context) {
            Ok(v) => v.join(" / "),
            Err(e) => format!("Invalid pointer target: {}", e),
        };

        Ok(format!("(ref) {} => {}", pointer_display, target_display))
    }

    pub fn size(&self) -> u64 {
        self.number_format.size()
    }

    pub fn related(&self, context: &H2Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![
            (self.number_format.read(context)?.to_index()?, *self.target_type.clone())
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    use crate::datatype::helpers::H2Context;
    use crate::datatype::basic::h2integer::H2Integer;
    use crate::datatype::composite::h2array::H2Array;
    use crate::datatype::helpers::sized_number::{SizedNumber, NumberFormat, SizedDisplay};

    #[test]
    fn test_pointer() -> SimpleResult<()> {
        let data = b"\x00\x08AAAAAA\x00\x01\x02\x03".to_vec();
        let context = H2Context::new(&data);

        let t: H2Type = H2Pointer::new(
            NumberFormat::U16_BIG,
            SizedDisplay::Hex,
            H2Integer::new(NumberFormat::U32_BIG, SizedDisplay::Hex).into()
        ).into();

        assert_eq!(2, t.size());

        println!("Type: {:?}", t);
        println!("\nto_strings:\n{}", t.to_strings(&context)?.join("\n"));

        Ok(())
    }

    #[test]
    fn test_complex_pointer() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x08\x00\x00\x00\x10AABBCCDD\x00\x01\x02\x03\x04\x05\x06\x07\x08".to_vec();
        let context = H2Context::new(&data);

        let t: H2Type = H2Array::new(2,
            H2Pointer::new(NumberFormat::U32_BIG, SizedDisplay::Hex,
                H2Array::new(4,
                    H2Integer::new(
                        NumberFormat::U16_BIG, SizedDisplay::Hex,
                    ).into()
                ).into()
            ).into()
        ).into();

        assert_eq!(8, t.size());

        println!("Type: {:?}", t);
        println!("\nto_strings:\n{}", t.to_strings(&context)?.join("\n"));

        Ok(())
    }
}
