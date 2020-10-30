use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use sized_number::{SizedDefinition, SizedDisplay};

use crate::datatype::{H2Type, H2Types, H2TypeTrait, ResolveOffset, AlignValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Pointer {
    definition: SizedDefinition,
    display: SizedDisplay,

    target_type: Box<H2Type>,
}

impl H2Pointer {
    pub fn new_aligned(alignment: AlignValue, definition: SizedDefinition, display: SizedDisplay, target_type: H2Type) -> H2Type {
        H2Type::new(alignment, H2Types::H2Pointer(Self {
            definition: definition,
            display: display,
            target_type: Box::new(target_type),
        }))
    }

    pub fn new(definition: SizedDefinition, display: SizedDisplay, target_type: H2Type) -> H2Type {
        Self::new_aligned(AlignValue::None, definition, display, target_type)
    }
}

impl H2TypeTrait for H2Pointer {
    fn is_static(&self) -> bool {
        true
    }

    fn size(&self, _offset: &ResolveOffset) -> SimpleResult<u64> {
        Ok(self.definition.size())
    }

    fn to_string(&self, offset: &ResolveOffset) -> SimpleResult<String> {
        match offset {
            ResolveOffset::Static(_) => Ok(format!("Pointer to {}", self.target_type.to_string(offset)?)),
            ResolveOffset::Dynamic(context) => {
                // Read the current value
                let target_offset = self.definition.to_u64(context)?;
                let pointer_display = self.definition.to_string(context, self.display)?;

                // Read the target from a separate context
                let target = ResolveOffset::from(context.at(target_offset));
                let target_display = match self.target_type.to_string(&target) {
                    Ok(v) => v,
                    Err(e) => format!("Invalid pointer target: {}", e),
                };

                Ok(format!("(ref) {} => {}", pointer_display, target_display))
            }
        }
    }

    fn related(&self, offset: &ResolveOffset) -> SimpleResult<Vec<(u64, H2Type)>> {
        match offset {
            ResolveOffset::Static(_) => bail!("Cannot get related statically"),
            ResolveOffset::Dynamic(context) => {
                Ok(vec![
                    (self.definition.to_u64(context)?, *self.target_type.clone())
                ])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, Endian};

    use crate::datatype::basic_type::h2number::H2Number;

    #[test]
    fn test_pointer() -> SimpleResult<()> {
        let data = b"\x00\x08AAAAAA\x00\x01\x02\x03".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        // 16-bit big-endian pointer (0x0008) that displays as hex
        let t = H2Pointer::new(
            SizedDefinition::U16(Endian::Big),
            SizedDisplay::Hex(Default::default()),

            // ...pointing to a 32-bit big-endian number (0x00010203)
            H2Number::new(
                SizedDefinition::U32(Endian::Big),
                SizedDisplay::Hex(Default::default()),
            )
        );

        // A 16-bit pointer is 2 bytes
        assert_eq!(2, t.actual_size(&s_offset).unwrap());
        assert_eq!(2, t.actual_size(&d_offset).unwrap());

        // Make sure it resolves the other variable
        assert!(t.to_string(&d_offset)?.starts_with("(ref) 0x0008"));

        // It has one related value - the int it points to
        assert!(t.related(&s_offset).is_err());
        assert_eq!(1, t.related(&d_offset)?.len());

        Ok(())
    }

    #[test]
    fn test_nested_pointer() -> SimpleResult<()> {
        //           -P1-  --P2-- -----P3--------
        let data = b"\x01\x00\x03\x07\x00\x00\x00ABCDEFGH".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        let hex_display = SizedDisplay::Hex(Default::default());

        let t = H2Pointer::new(SizedDefinition::U8, hex_display, // P1
            H2Pointer::new(SizedDefinition::U16(Endian::Big), hex_display, // P2
                H2Pointer::new(SizedDefinition::U32(Endian::Little), hex_display, // P3
                    H2Number::new(SizedDefinition::U64(Endian::Big), hex_display),
                )
            )
        );

        assert_eq!(1, t.actual_size(&s_offset).unwrap());
        assert_eq!(1, t.actual_size(&d_offset).unwrap());

        assert_eq!(1, t.related(&d_offset)?.len());
        assert!(t.to_string(&d_offset)?.ends_with("0x4142434445464748"));

        Ok(())
    }
}
