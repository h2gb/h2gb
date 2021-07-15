//! Some good info should go here!

use redo::Record;
use simple_error::{SimpleResult, bail};

use crate::actions::*;
use crate::transformation::{TransformBlockCipher, BlockCipherType, BlockCipherMode, BlockCipherPadding};
use crate::datatype::{H2Type, ResolvedType};
use crate::datatype::simple::H2Number;
use crate::datatype::composite::H2Struct;
use crate::datatype::composite::string::LPString;
use crate::generic_number::{GenericNumber, GenericReader, Endian, EnumFormatter, EnumType, DefaultFormatter};

const TERRARIA_KEY: &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";
const TERRARIA_IV:  &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";

const OFFSET_SPAWN_POINTS: usize = 0x99c;
const OFFSET_JOURNEY_DATA: usize = 0x6b;

pub fn create_entry(record: &mut Record<Action>, buffer: &str, layer: &str, datatype: H2Type, offset: usize, comment: Option<&str>) -> SimpleResult<ResolvedType> {
    // Create the entry
    let create_action = ActionEntryCreateAndInsert::new(buffer, layer, datatype, offset);
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

pub fn peek_entry(record: &mut Record<Action>, buffer: &str, datatype: H2Type, offset: usize) -> SimpleResult<ResolvedType> {
    Ok(record.target().entry_create(buffer, datatype, offset)?.resolved().to_owned())
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
        H2Number::new(GenericReader::U16(Endian::Little), EnumFormatter::new(EnumType::TerrariaVersion)),
        0x00, // Offset
        Some("Version number"),
    );

    // Create an entry for the name
    let name = create_entry(
        record,
        buffer,
        "default",
        LPString::new(
            H2Number::new(GenericReader::U8, DefaultFormatter::new()),
            H2Number::new(GenericReader::ASCII, DefaultFormatter::new()),
        )?,
        0x18, // Offset
        Some("Character name"),
    )?;

    // Create an entry for the game mode
    let _game_mode = create_entry(
        record,
        buffer,
        "default",
        H2Number::new(GenericReader::U8, EnumFormatter::new(EnumType::TerrariaGameMode)),
        name.actual_range.end as usize, // Offset
        Some("Game mode"),
    )?;

    // Get the offset to research data, which is a static offset from the end
    // of name
    let mut current_spawn_offset = name.actual_range.end as usize + OFFSET_SPAWN_POINTS;

    // This defines a spawnpoint entry, and is used for each spawnpoint
    let spawn_entry = H2Struct::new(vec![
        ("x".to_string(),     H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
        ("y".to_string(),     H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
        ("seed".to_string(),  H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
        ("world".to_string(), LPString::new(
            H2Number::new(GenericReader::U8, DefaultFormatter::new()),
            H2Number::new(GenericReader::ASCII, DefaultFormatter::new()),
        )?),
    ])?;

    loop {
        // Check for the terminator
        let terminator_type = H2Number::new(GenericReader::I32(Endian::Little), DefaultFormatter::new());
        let possible_terminator = peek_entry(record, buffer, terminator_type.clone(), current_spawn_offset)?;
        if let Some(n) = possible_terminator.as_number {
            if n == GenericNumber::from(-1) {
                create_entry(record, buffer, "default", terminator_type, current_spawn_offset, Some("Spawn point sentinel value (terminator)"))?;
                break;
            }
        }
        // if let Some(possible_terminator.as_number) == GenericNumber::from(-1) {
        // }

        let spawn_point = create_entry(
            record,
            buffer,
            "default",
            spawn_entry.clone(),
            current_spawn_offset,
            Some("Spawn point"),
        )?;

        // Update to the next spawn offset
        current_spawn_offset = spawn_point.actual_range.end as usize;
    }

    let journeymode_item_entry = H2Struct::new(vec![
        ("item".to_string(), LPString::new(
            H2Number::new(GenericReader::U8, DefaultFormatter::new()),
            H2Number::new(GenericReader::ASCII, DefaultFormatter::new()),
        )?),

        ("quantity".to_string(),  H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
    ])?;

    // TODO: Is journey mode?
    let mut current_journey_offset = current_spawn_offset + OFFSET_JOURNEY_DATA;
    loop {
        let terminator_type = H2Number::new(GenericReader::U8, DefaultFormatter::new());
        let possible_terminator = peek_entry(record, buffer, terminator_type.clone(), current_journey_offset)?;
        if let Some(n) = possible_terminator.as_number {
            if n == GenericNumber::from(0u8) {
                create_entry(record, buffer, "default", terminator_type, current_journey_offset, Some("Journey mode entry sentinel value (terminator)"))?;
                break;
            }
        }

        let journey_item = create_entry(
            record,
            buffer,
            "default",
            journeymode_item_entry.clone(),
            current_journey_offset,
            Some("Journeymode item"),
        )?;

        // Update to the next journey offset
        current_journey_offset = journey_item.actual_range.end as usize;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::PathBuf;

    //use pretty_assertions::assert_eq;

    use crate::project::H2Project;
    use crate::actions::ActionBufferCreateFromBytes;


    #[test]
    fn test_analyze() -> SimpleResult<()> {
        // Load the data
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/terraria/ManySpawnPoints.plr");

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
