//! Some good info should go here!

use redo::Record;
use simple_error::{SimpleResult, bail};

use crate::actions::*;
//use crate::project::h2project::H2Project;
use crate::transformation::{TransformBlockCipher, BlockCipherType, BlockCipherMode, BlockCipherPadding};
use crate::datatype::{H2Type, H2Number, LPString, ASCII, StrictASCII, ResolvedType};
use crate::sized_number::{SizedDefinition, SizedDisplay, Endian, EnumType};

const TERRARIA_KEY: &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";
const TERRARIA_IV:  &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";

pub fn create_entry(record: &mut Record<Action>, buffer: &str, layer: &str, datatype: H2Type, offset: usize, comment: Option<&str>) -> SimpleResult<ResolvedType> {
    // Create the entry
    let create_action = ActionEntryCreateFromType::new(buffer, layer, datatype, offset);
    record.apply(create_action)?;

    // Add a comment
    if let Some(c) = comment {
        let comment_action = ActionEntrySetComment::new(buffer, layer, offset, Some(c.to_string()));
        record.apply(comment_action)?;
    }

    // Retrieve and return the entry
    match record.target().entry_get(buffer, layer, offset) {
        Some(entry) => Ok(entry.resolved().clone()),
        None => bail!("Entry didn't correctly insert"),
    }
}

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
    let _version = create_entry(
        record,
        buffer,
        "default",
        H2Number::new(SizedDefinition::U16(Endian::Little), SizedDisplay::Decimal(Default::default())),
        0x00, // Offset
        Some("Version number"),
    );

    // Add a comment to the version
    record.apply(ActionEntrySetComment::new(buffer, "default", 0, Some("Version number".to_string())))?;


    // Create an entry for the name
    let name = create_entry(
        record,
        buffer,
        "default",
        LPString::new(
            H2Number::new(SizedDefinition::U8, SizedDisplay::Decimal(Default::default())),
            ASCII::new(StrictASCII::Permissive),
        )?,
        0x18, // Offset
        Some("Character name"),
    )?;

    // Create an entry for the game mode
    let _game_mode = create_entry(
        record,
        buffer,
        "default",
        H2Number::new(SizedDefinition::U8, SizedDisplay::Enum(EnumType::TerrariaGameMode)),
        name.actual_range.end as usize, // Offset
        Some("Game mode"),
    )?;

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
