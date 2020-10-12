use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use sized_number::{Context, SizedDefinition, SizedDisplay};

use crate::datatype::H2Type;
use crate::datatype::basic::H2BasicType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Pointer {
    definition: SizedDefinition,
    display: SizedDisplay,

    target_type: Box<H2Type>,
}

impl From<H2Pointer> for H2Type {
    fn from(o: H2Pointer) -> H2Type {
        H2Type::from(H2BasicType::Pointer(o))
    }
}

impl H2Pointer {
    pub fn new(definition: SizedDefinition, display: SizedDisplay, target_type: H2Type) -> SimpleResult<Self> {
        if !definition.can_be_u64() {
            bail!("H2Pointer's definition must be an unsigned value no more than 64 bits / 8 bytes");
        }

        Ok(H2Pointer {
            definition: definition,
            display: display,

            target_type: Box::new(target_type),
        })
    }

    pub fn to_u64(&self, context: &Context) -> SimpleResult<u64> {
        self.definition.to_u64(context)
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        // Read the current value
        let target_offset = self.to_u64(context)?;
        let pointer_display = self.definition.to_string(context, self.display)?;

        // Read the target from a separate context
        let mut target_context = context.clone();
        target_context.set_position(target_offset);
        let target_display = match self.target_type.to_strings(&target_context) {
            Ok(v) => v.join(" / "),
            Err(e) => format!("Invalid pointer target: {}", e),
        };

        Ok(format!("(ref) {} => {}", pointer_display, target_display))
    }

    pub fn size(&self) -> u64 {
        self.definition.size()
    }

    pub fn related(&self, context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![
            (self.to_u64(context)?, *self.target_type.clone())
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Endian;

    use crate::datatype::basic::h2integer::H2Integer;
    use crate::datatype::composite::h2array::H2Array;

    #[test]
    fn test_pointer() -> SimpleResult<()> {
        let data = b"\x00\x08AAAAAA\x00\x01\x02\x03".to_vec();
        let context = Context::new(&data);

        let t: H2Type = H2Pointer::new(
            SizedDefinition::U16(Endian::Big),
            SizedDisplay::Hex(Default::default()),

            H2Integer::new(
                SizedDefinition::U32(Endian::Big),
                SizedDisplay::Hex(Default::default()),
            ).into()
        )?.into();

        assert_eq!(2, t.size());

        println!("Type: {:?}", t);
        println!("\nto_strings:\n{}", t.to_strings(&context)?.join("\n"));

        Ok(())
    }

    #[test]
    fn test_complex_pointer() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x08\x00\x00\x00\x10AABBCCDD\x00\x01\x02\x03\x04\x05\x06\x07\x08".to_vec();
        let context = Context::new(&data);

        let t: H2Type = H2Array::new(2,
            H2Pointer::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default()),
                H2Array::new(4,
                    H2Integer::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default())).into()
                ).into()
            )?.into()
        ).into();

        assert_eq!(8, t.size());

        println!("Type: {:?}", t);
        println!("\nto_strings:\n{}", t.to_strings(&context)?.join("\n"));

        Ok(())
    }
}
