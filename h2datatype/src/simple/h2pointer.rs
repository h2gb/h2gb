//use serde::{Serialize, Deserialize};
//use simple_error::SimpleResult;

//use generic_number::{IntegerReader, GenericFormatter};
//use crate::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

///// Defines a pointer type - a numeric type that points to another location.
/////
///// This is defined very similarly to [`crate::simple::numeric::H2Integer`],
///// with one additional field: the `target_type`, which is the type of the value
///// that the pointer points to.
//#[derive(Debug, Clone, Serialize, Deserialize)]
//pub struct H2Pointer {
//    definition: IntegerReader,
//    display: GenericFormatter,

//    target_type: Box<H2Type>,
//}

//impl H2Pointer {
//    pub fn new_aligned(alignment: Alignment, definition: IntegerReader, display: GenericFormatter, target_type: H2Type) -> H2Type {
//        // TODO: Ensure the definition can be a u64
//        H2Type::new(alignment, H2Types::H2Pointer(Self {
//            definition: definition,
//            display: display,
//            target_type: Box::new(target_type),
//        }))
//    }

//    pub fn new(definition: IntegerReader, display: GenericFormatter, target_type: H2Type) -> H2Type {
//        Self::new_aligned(Alignment::None, definition, display, target_type)
//    }
//}

//impl H2TypeTrait for H2Pointer {
//    fn base_size(&self, offset: Offset) -> SimpleResult<u64> {
//        // TODO: I'm not sure if using the static size here is really something I should care about, as opposed to just reading + checking
//        match self.definition.size() {
//            Some(v) => Ok(v as u64),
//            None    => Ok(self.definition.read(offset.get_dynamic()?)?.size() as u64),
//        }
//    }

//    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
//        match offset {
//            Offset::Static(_) => Ok(format!("Pointer to {}", self.target_type.to_display(offset)?)),
//            Offset::Dynamic(context) => {
//                // Read the current value
//                let target_offset = self.definition.read(context)?;
//                let pointer_display = self.display.render(target_offset)?;

//                // Read the target from a separate context
//                let target = Offset::from(context.at(target_offset.as_u64()?));
//                let target_display = match self.target_type.to_display(target) {
//                    Ok(v) => v,
//                    Err(e) => format!("Invalid pointer target: {}", e),
//                };

//                Ok(format!("(ref) {} => {}", pointer_display, target_display))
//            }
//        }
//    }

//    fn related(&self, offset: Offset) -> SimpleResult<Vec<(u64, H2Type)>> {
//        let context = offset.get_dynamic()?;
//        let target = self.definition.read(context)?;

//        Ok(vec![
//            (target.as_u64()?, *self.target_type.clone())
//        ])
//    }
//}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use simple_error::SimpleResult;

//    use generic_number::{Context, Endian, HexFormatter};
//    use crate::simple::numeric::H2Integer;

//    #[test]
//    fn test_pointer() -> SimpleResult<()> {
//        let data = b"\x00\x08AAAAAA\x00\x01\x02\x03".to_vec();
//        let s_offset = Offset::Static(0);
//        let d_offset = Offset::Dynamic(Context::new(&data));

//        // 16-bit big-endian pointer (0x0008) that displays as hex
//        let t = H2Pointer::new(
//            IntegerReader::U16(Endian::Big),
//            HexFormatter::pretty(),

//            // ...pointing to a 32-bit big-endian number (0x00010203)
//            H2Integer::new(
//                IntegerReader::U32(Endian::Big),
//                HexFormatter::pretty_string(),
//            )
//        );

//        // A 16-bit pointer is 2 bytes
//        assert_eq!(2, t.base_size(s_offset).unwrap());
//        assert_eq!(2, t.base_size(d_offset).unwrap());

//        // Make sure it resolves the other variable
//        assert!(t.to_display(d_offset)?.starts_with("(ref) 0x0008"));

//        // It has one related value - the int it points to
//        assert!(t.related(s_offset).is_err());
//        assert_eq!(1, t.related(d_offset)?.len());

//        Ok(())
//    }

//    #[test]
//    fn test_nested_pointer() -> SimpleResult<()> {
//        //           -P1-  --P2-- -----P3--------
//        let data = b"\x01\x00\x03\x07\x00\x00\x00ABCDEFGH".to_vec();
//        let s_offset = Offset::Static(0);
//        let d_offset = Offset::Dynamic(Context::new(&data));

//        let t = H2Pointer::new(IntegerReader::U8, HexFormatter::pretty(), // P1
//            H2Pointer::new(IntegerReader::U16(Endian::Big), HexFormatter::pretty(), // P2
//                H2Pointer::new(IntegerReader::U32(Endian::Little), HexFormatter::pretty(), // P3
//                    H2Integer::new(IntegerReader::U64(Endian::Big), HexFormatter::pretty()),
//                )
//            )
//        );

//        assert_eq!(1, t.base_size(s_offset).unwrap());
//        assert_eq!(1, t.base_size(d_offset).unwrap());

//        assert_eq!(1, t.related(d_offset)?.len());
//        assert!(t.to_display(d_offset)?.ends_with("0x4142434445464748"));

//        Ok(())
//    }
//}
