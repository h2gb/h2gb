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
        H2Type::from(H2BasicType::from(o))
    }
}

impl From<H2Pointer> for H2BasicType {
    fn from(o: H2Pointer) -> H2BasicType {
        H2BasicType::Pointer(o)
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

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        // Read the current value
        let target_offset = self.definition.to_u64(context)?;
        let pointer_display = self.definition.to_string(context, self.display)?;

        // Read the target from a separate context
        let target_context = context.at(target_offset);
        let target_display = match self.target_type.to_string(&target_context) {
            Ok(v) => v,
            Err(e) => format!("Invalid pointer target: {}", e),
        };

        Ok(format!("(ref) {} => {}", pointer_display, target_display))
    }

    pub fn size(&self) -> u64 {
        self.definition.size()
    }

    pub fn related(&self, context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![
            (self.definition.to_u64(context)?, *self.target_type.clone())
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Endian;

    use crate::datatype::basic::h2number::H2Number;

    #[test]
    fn test_pointer() -> SimpleResult<()> {
        let data = b"\x00\x08AAAAAA\x00\x01\x02\x03".to_vec();
        let context = Context::new(&data);

        // 16-bit big-endian pointer (0x0008) that displays as hex
        let t = H2Pointer::new(
            SizedDefinition::U16(Endian::Big),
            SizedDisplay::Hex(Default::default()),

            // ...pointing to a 32-bit big-endian number (0x00010203)
            H2Number::new(
                SizedDefinition::U32(Endian::Big),
                SizedDisplay::Hex(Default::default()),
            ).into()
        )?;

        // A 16-bit pointer is 2 bytes
        assert_eq!(2, t.size());
        assert!(t.to_string(&context)?.starts_with("(ref) 0x0008"));

        // It has one related value - the int it points to
        let r = t.related(&context)?;
        assert_eq!(1, r.len());

        Ok(())
    }

    #[test]
    fn test_nested_pointer() -> SimpleResult<()> {
        //           -P1-  --P2-- -----P3--------
        let data = b"\x01\x00\x03\x07\x00\x00\x00ABCDEFGH".to_vec();
        let context = Context::new(&data);

        let hex_display = SizedDisplay::Hex(Default::default());

        let t = H2Pointer::new(SizedDefinition::U8, hex_display, // P1
            H2Pointer::new(SizedDefinition::U16(Endian::Big), hex_display, // P2
                H2Pointer::new(SizedDefinition::U32(Endian::Little), hex_display, // P3
                    H2Number::new(SizedDefinition::U64(Endian::Big), hex_display).into(),
                )?.into(),
            )?.into(),
        )?;

        assert_eq!(1, t.size());
        assert_eq!(1, t.related(&context)?.len());
        assert!(t.to_string(&context)?.ends_with("0x4142434445464748"));

        Ok(())
    }
}
