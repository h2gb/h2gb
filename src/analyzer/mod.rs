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
use crate::generic_number::{GenericNumber, GenericReader, Endian, EnumFormatter, EnumType, DefaultFormatter, HexFormatter, BooleanFormatter};

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

    static ref INVENTORY_ITEM: H2Type = {
        H2Struct::new(vec![
            ("id".to_string(),          H2Number::new(GenericReader::U32(Endian::Little), EnumFormatter::new(EnumType::TerrariaItem))),
            ("quantity".to_string(),    H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
            ("affix".to_string(),       H2Number::new(GenericReader::U8, HexFormatter::pretty())),
            ("is_favorite".to_string(), H2Number::new(GenericReader::U8, BooleanFormatter::new())),
        ]).unwrap()
    };

    static ref STORED_ITEM: H2Type = {
        H2Struct::new(vec![
            ("id".to_string(),          H2Number::new(GenericReader::U32(Endian::Little), EnumFormatter::new(EnumType::TerrariaItem))),
            ("quantity".to_string(),    H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
            ("affix".to_string(),       H2Number::new(GenericReader::U8, HexFormatter::pretty())),
        ]).unwrap()
    };
}

struct TerrariaOffsets {
    name:           usize,

    // Relative to name
    game_mode:      usize,
    col:            usize,
    inventory:      usize,
    coins_and_ammo: usize,
    piggy_bank:     usize,
    safe:           usize,
    spawnpoints:    usize,
    buffs:          Option<usize>,

    // Relative to end of spawnpoints
    journey_data:   Option<usize>,
}

fn get_terraria_offsets(version: u64) -> TerrariaOffsets {
    if version < 230 {
        TerrariaOffsets {
            // Offset from start of file
            name:           0x18,

            // Offset from end of name
            game_mode:      0x00,
            col:            0x28,
            inventory:      0xd3,
            coins_and_ammo: 0x2c7,
            piggy_bank:     0x349,
            safe:           0x4b1,
            buffs:          None,
            spawnpoints:    0x99a, // Not tested, likely wrong

            // No JourneyMode
            journey_data:   None,
        }
    } else {
        TerrariaOffsets {
            // Offset from start of file
            name:           0x18,

            // Offset from end of name
            game_mode:      0x00,
            col:            0x2a,
            inventory:      0xd5,
            coins_and_ammo: 0x2c9,
            piggy_bank:     0x34b,
            safe:           0x4b3,
            buffs:          Some(0x8ec),
            spawnpoints:    0x99c,

            // Offset from the end of spawnpoints
            journey_data:   Some(0x6b),
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

fn parse_game_mode(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<ResolvedType> {
    create_entry(
        record,
        buffer,
        LAYER,
        &H2Number::new(GenericReader::U8, EnumFormatter::new(EnumType::TerrariaGameMode)),
        offset,
        Some("Game mode"),
    )
}

fn parse_inventory(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    add_comment(record, buffer, LAYER, offset,  "Start offset for inventory")?;
    // Technically this is an array, but we don't really handle arrays well enough to use one
    for i in (offset..(offset + 500)).step_by(10) {
        create_entry(
            record,
            buffer,
            LAYER,
            &*INVENTORY_ITEM,
            i,
            None,
        )?;
    }
    add_comment(record, buffer, LAYER, offset + 500 - 1, "End offset for inventory")?;

    Ok(())
}

fn parse_coins_and_ammo(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    add_comment(record, buffer, LAYER, offset, "Start offset for coins_and_ammo")?;

    for i in (offset..(offset + 80)).step_by(10) {
        create_entry(
            record,
            buffer,
            LAYER,
            &*INVENTORY_ITEM,
            i,
            None,
        )?;
    }

    add_comment(record, buffer, LAYER, offset + 80 - 1, "End offset for coins_and_ammo")?;

    Ok(())
}

fn parse_piggy_bank(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    add_comment(record, buffer, LAYER, offset,  "Start offset for piggy bank")?;
    // Technically this is an array, but we don't really handle arrays well enough to use one
    for i in (offset..(offset + 360)).step_by(9) {
        create_entry(
            record,
            buffer,
            LAYER,
            &*STORED_ITEM,
            i,
            None,
        )?;
    }
    add_comment(record, buffer, LAYER, offset + 360 - 1, "End offset for piggy bank")?;

    Ok(())
}

fn parse_safe(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    add_comment(record, buffer, LAYER, offset,  "Start offset for safe")?;
    // Technically this is an array, but we don't really handle arrays well enough to use one
    for i in (offset..(offset + 360)).step_by(9) {
        create_entry(
            record,
            buffer,
            LAYER,
            &*STORED_ITEM,
            i,
            None,
        )?;
    }
    add_comment(record, buffer, LAYER, offset + 360 - 1, "End offset for safe")?;

    Ok(())
}

fn parse_spawnpoints(record: &mut Record<Action>, buffer: &str, starting_offset: usize) -> SimpleResult<usize> {
    let mut current_spawn_offset = starting_offset;
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

    Ok(current_spawn_offset)
}

fn parse_journeymode(record: &mut Record<Action>, buffer: &str, starting_offset: usize) -> SimpleResult<()> {
    let mut current_journey_offset = starting_offset;

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
    let game_mode = parse_game_mode(record, buffer, base_offset + offsets.game_mode)?.as_number.ok_or(
        SimpleError::new("Game mode could not be parsed properly (could not be represented as a number)")
    )?.as_u64().map_err( |e| SimpleError::new(format!("Game mode could not be parsed properly (could not be interpreted as a u64): {:?}", e)))?;

    // Comment the other offsets because I don't know how to parse them yet
    add_comment(record, buffer, LAYER, base_offset + offsets.col,        "Offset for 'col' (I'm not actually sure what that is.. maybe ammo?)")?;
    parse_inventory(record, buffer, base_offset + offsets.inventory)?;
    parse_coins_and_ammo(record, buffer, base_offset + offsets.coins_and_ammo)?;
    parse_piggy_bank(record, buffer, base_offset + offsets.piggy_bank)?;
    parse_safe(record, buffer, base_offset + offsets.safe)?;

    if let Some(offset_buffs) = offsets.buffs {
        add_comment(record, buffer, LAYER, base_offset + offset_buffs,  "Offset for buffs")?;
    }

    // Parse the spawnpoints
    let new_base_offset = parse_spawnpoints(record, buffer, base_offset + offsets.spawnpoints)?;

    // game_mode 3 == Journey Mode
    if game_mode == 3 {
        // Make sure we have the correct offset
        if let Some(offset) = offsets.journey_data {
            parse_journeymode(record, buffer, new_base_offset + offset)?;
        }
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
        // d.push("testdata/terraria/TestChar.plr");
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
