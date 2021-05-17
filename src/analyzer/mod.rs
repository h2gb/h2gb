//! Some good info should go here!

use redo::Record;
use simple_error::SimpleResult;

use crate::actions::*;
//use crate::project::h2project::H2Project;
use crate::transformation::{TransformBlockCipher, BlockCipherType, BlockCipherMode, BlockCipherPadding};
use crate::datatype::{H2Number, LPString, ASCII, StrictASCII};
use crate::sized_number::{SizedDefinition, SizedDisplay, Endian, EnumType};

const TERRARIA_KEY: &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";
const TERRARIA_IV:  &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";

pub fn analyze_terraria(record: &mut Record<Action>, buffer: &str) -> SimpleResult<()> {
    // Transform -> decrypt
    let transformation = TransformBlockCipher::new(
        BlockCipherType::AES,
        BlockCipherMode::CBC,
        BlockCipherPadding::Pkcs7,
        TERRARIA_KEY.to_vec(),
        Some(TERRARIA_IV.to_vec()),
    )?;
    record.apply(ActionBufferTransform::new(buffer, transformation))?;

    // Create a layer
    record.apply(ActionLayerCreate::new(buffer, "default"))?;

    // Create an entry for the version
    let datatype = H2Number::new(SizedDefinition::U16(Endian::Little), SizedDisplay::Decimal);
    record.apply(ActionEntryCreateFromType::new(buffer, "default", datatype, 0))?;

    //let entry = record.target().entry_get(buffer, "default", 0).unwrap().resolved();
    record.apply(ActionEntrySetComment::new(buffer, "default", 0, Some("Version number".to_string())))?;


    // Create an entry for the name
    let datatype = LPString::new(
        H2Number::new(SizedDefinition::U8, SizedDisplay::Decimal),
        ASCII::new(StrictASCII::Permissive),
    )?;
    record.apply(ActionEntryCreateFromType::new(buffer, "default", datatype, 0x18))?;

    // Find the end of the name
    let name_entry = record.target().entry_get(buffer, "default", 0x18).unwrap().resolved();
    let name_datatype = H2Number::new(SizedDefinition::U8, SizedDisplay::Enum(EnumType::TerrariaGameMode));
    record.apply(ActionEntryCreateFromType::new(buffer, "default", name_datatype, (name_entry.actual_range.end) as usize))?;



    // Create entries:
    // -> Version -> 16 bits little endian
    //    -> Check if it's a version we can handle
    // -> Name -> length-prefixed string from NAME_OFFSET
    // -> Game mode -> 8 bits right after name
    // ... other stuff

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::PathBuf;

    //use pretty_assertions::assert_eq;

    use crate::project::h2project::H2Project;
    use crate::actions::ActionBufferCreateFromBytes;


    #[test]
    fn test_analyze() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/terraria/TestChar.plr");

        // Create a fresh record
        let mut record: Record<Action> = Record::new(
            H2Project::new("Terraria Test", "1.0")
        );

        // Load the file data into a new buffer
        let data = fs::read(d).unwrap();
        let action = ActionBufferCreateFromBytes::new("buffer", &data, 0x0);
        record.apply(action)?;

        analyze_terraria(&mut record, "buffer")?;

        println!("{}", &record.target());
        Ok(())
    }
}
