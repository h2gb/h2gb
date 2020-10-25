use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use sized_number::{Context, SizedDefinition, SizedDisplay};

use crate::datatype::{H2Type, H2Types, H2TypeTrait};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Pointer {
    definition: SizedDefinition,
    display: SizedDisplay,

    target_type: Box<H2Type>,
}

impl From<H2Pointer> for H2Type {
    fn from(o: H2Pointer) -> H2Type {
        H2Type::new(H2Types::H2Pointer(o))
    }
}

impl From<(H2Pointer, u64)> for H2Type {
    fn from(o: (H2Pointer, u64)) -> H2Type {
        H2Type::new_aligned(H2Types::H2Pointer(o.0), Some(o.1))
    }
}

impl H2Pointer {
    pub fn new(definition: SizedDefinition, display: SizedDisplay, target_type: H2Type) -> Self {
        Self {
            definition: definition,
            display: display,
            target_type: Box::new(target_type),
        }
    }
}

impl H2TypeTrait for H2Pointer {
    fn is_static(&self) -> bool {
        true
    }

    fn static_size(&self) -> Option<u64> {
        Some(self.definition.size())
    }

    fn name(&self) -> String {
        "Pointer".to_string()
    }

    fn to_string(&self, context: &Context) -> SimpleResult<String> {
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

    fn related(&self, context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
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

    use crate::datatype::basic_type::h2number::H2Number;

    #[test]
    fn test_pointer() -> SimpleResult<()> {
        let data = b"\x00\x08AAAAAA\x00\x01\x02\x03".to_vec();
        let context = Context::new(&data);

        // 16-bit big-endian pointer (0x0008) that displays as hex
        let t = H2Type::from(H2Pointer::new(
            SizedDefinition::U16(Endian::Big),
            SizedDisplay::Hex(Default::default()),

            // ...pointing to a 32-bit big-endian number (0x00010203)
            H2Type::from(H2Number::new(
                SizedDefinition::U32(Endian::Big),
                SizedDisplay::Hex(Default::default()),
            ))
        ));

        // A 16-bit pointer is 2 bytes
        assert_eq!(2, t.static_size().unwrap());
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

        let t = H2Type::from(H2Pointer::new(SizedDefinition::U8, hex_display, // P1
            H2Type::from(H2Pointer::new(SizedDefinition::U16(Endian::Big), hex_display, // P2
                H2Type::from(H2Pointer::new(SizedDefinition::U32(Endian::Little), hex_display, // P3
                    H2Type::from(H2Number::new(SizedDefinition::U64(Endian::Big), hex_display)),
                ))
            ))
        ));

        assert_eq!(1, t.static_size().unwrap());
        assert_eq!(1, t.related(&context)?.len());
        assert!(t.to_string(&context)?.ends_with("0x4142434445464748"));

        Ok(())
    }
}
