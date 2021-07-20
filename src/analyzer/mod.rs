//! Some good info should go here!

use redo::Record;
use simple_error::{SimpleResult, SimpleError};
use lazy_static::lazy_static;

use crate::actions::*;
use crate::transformation::{Transformation, TransformBlockCipher, BlockCipherType, BlockCipherMode, BlockCipherPadding};
use crate::datatype::{H2Type, ResolvedType};
use crate::datatype::simple::H2Number;
use crate::datatype::composite::H2Struct;
use crate::datatype::composite::string::LPString;
use crate::generic_number::{GenericNumber, GenericReader, Endian, EnumFormatter, EnumType, DefaultFormatter};

mod helpers;
use helpers::*;

const LAYER: &'static str = "default";

const TERRARIA_KEY: &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";
const TERRARIA_IV:  &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";

const OFFSET_SPAWN_POINTS: usize = 0x99c;
const OFFSET_JOURNEY_DATA: usize = 0x6b;

lazy_static! {
    static ref TRANSFORMATION_DECRYPT: Transformation = {
        TransformBlockCipher::new(
            BlockCipherType::AES,
            BlockCipherMode::CBC,
            BlockCipherPadding::Pkcs7,
            TERRARIA_KEY.to_vec(),
            Some(TERRARIA_IV.to_vec()),
        ).unwrap()
    };

    static ref TERRARIA_LPSTRING: H2Type = {
        LPString::new(
            H2Number::new(GenericReader::U8, DefaultFormatter::new()),
            H2Number::new(GenericReader::ASCII, DefaultFormatter::new()),
        ).unwrap()
    };

    static ref SPAWNPOINT_ENTRY: H2Type = {
        H2Struct::new(vec![
            ("x".to_string(),     H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
            ("y".to_string(),     H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
            ("seed".to_string(),  H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
            ("world".to_string(), LPString::new(
                H2Number::new(GenericReader::U8, DefaultFormatter::new()),
                H2Number::new(GenericReader::ASCII, DefaultFormatter::new()),
            ).unwrap()),
        ]).unwrap()
    };

    static ref JOURNEYMODE_ITEM_ENTRY: H2Type = {
        H2Struct::new(vec![
            ("item".to_string(), LPString::new(
                H2Number::new(GenericReader::U8, DefaultFormatter::new()),
                H2Number::new(GenericReader::ASCII, DefaultFormatter::new()),
            ).unwrap()),

            ("quantity".to_string(),  H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
        ]).unwrap()
    };

}

struct TerrariaOffsets {
    name:         usize,
    col:          usize,
    inventory:    usize,
    coin:         usize,
    piggy_bank:   usize,
    safe:         usize,
    spawnpoints:  usize,
    buffs:        Option<usize>,
    journey_data: Option<usize>,
}

fn get_terraria_offsets(version: u64) -> TerrariaOffsets {
    if version < 230 {
        TerrariaOffsets {
            // Offset from start of file
            name:        0x18,

            // Offset from end of name
            col:         0x28,
            inventory:   0xd3,
            coin:        0x2c7,
            piggy_bank:  0x349,
            safe:        0x4b1,
            buffs:       None,
            spawnpoints: 0x99a, // Not tested, likely wrong

            // No JourneyMode
            journey_data: None,
        }
    } else {
        TerrariaOffsets {
            // Offset from start of file
            name:        0x18,

            // Offset from end of name
            col:         0x2a,
            inventory:   0xd5,
            coin:        0x2c9,
            piggy_bank:  0x34b,
            safe:        0x4b3,
            buffs:       Some(0x8ec),
            spawnpoints: 0x99c,

            // Offset from the end of spawnpoints
            journey_data: Some(0x6b),
        }
    }
}

fn transform_decrypt(record: &mut Record<Action>, buffer: &str) -> SimpleResult<()> {
    record.apply(ActionBufferTransform::new(buffer, *TRANSFORMATION_DECRYPT))
}

fn parse_version_number(record: &mut Record<Action>, buffer: &str) -> SimpleResult<ResolvedType> {
    create_entry(
        record,
        buffer,
        LAYER,
        &H2Number::new(GenericReader::U16(Endian::Little), EnumFormatter::new(EnumType::TerrariaVersion)),
        0x00, // Offset
        Some("Version number"),
    )
}

fn parse_name(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<ResolvedType> {
    create_entry(
        record,
        buffer,
        LAYER,
        &*TERRARIA_LPSTRING,
        offset,
        Some("Character name"),
    )
}

pub fn analyze_terraria(record: &mut Record<Action>, buffer: &str) -> SimpleResult<()> {
    // Decrypt the buffer
    transform_decrypt(record, buffer)?;

    // Create a layer
    record.apply(ActionLayerCreate::new(buffer, LAYER))?;

    // Create an entry for the version
    let version = parse_version_number(record, buffer)?;
    let version_number = version.as_number.ok_or(
        SimpleError::new("Could not parse the Terraria version number as a number")
    )?.as_u64()?;

    // Get the offsets for later
    let offsets = get_terraria_offsets(version_number);

    // Create an entry for the name
    let name = parse_name(record, buffer, offsets.name)?;

    // Get the offsets' base (end of name)
    let base_offset = name.actual_range.end as usize;

    // Create an entry for the game mode
    let _game_mode = create_entry(
        record,
        buffer,
        LAYER,
        &H2Number::new(GenericReader::U8, EnumFormatter::new(EnumType::TerrariaGameMode)),
        base_offset, // Offset
        Some("Game mode"),
    )?;

    // Get the offset to research data, which is a static offset from the end
    // of name
    let mut current_spawn_offset = base_offset + offsets.spawnpoints;

    // This defines a spawnpoint entry, and is used for each spawnpoint

    loop {
        // Check for the terminator
        let terminator_type = H2Number::new(GenericReader::I32(Endian::Little), DefaultFormatter::new());
        let possible_terminator = peek_entry(record, buffer, &terminator_type, current_spawn_offset)?;
        if let Some(n) = possible_terminator.as_number {
            if n == GenericNumber::from(-1) {
                create_entry(record, buffer, LAYER, &terminator_type, current_spawn_offset, Some("Spawn point sentinel value (terminator)"))?;
                break;
            }
        }
        // if let Some(possible_terminator.as_number) == GenericNumber::from(-1) {
        // }

        let spawn_point = create_entry(
            record,
            buffer,
            LAYER,
            &*SPAWNPOINT_ENTRY,
            current_spawn_offset,
            Some("Spawn point"),
        )?;

        // Update to the next spawn offset
        current_spawn_offset = spawn_point.actual_range.end as usize;
    }

    // TODO: Is journey mode?
    let mut current_journey_offset = current_spawn_offset + OFFSET_JOURNEY_DATA;
    loop {
        let terminator_type = H2Number::new(GenericReader::U8, DefaultFormatter::new());
        let possible_terminator = peek_entry(record, buffer, &terminator_type, current_journey_offset)?;
        if let Some(n) = possible_terminator.as_number {
            if n == GenericNumber::from(0u8) {
                create_entry(record, buffer, LAYER, &terminator_type, current_journey_offset, Some("Journey mode entry sentinel value (terminator)"))?;
                break;
            }
        }

        let journey_item = create_entry(
            record,
            buffer,
            LAYER,
            &*JOURNEYMODE_ITEM_ENTRY,
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
