//! So far, this is a simple demonstration of what we can do

use redo::Record;
use simple_error::SimpleResult;
use lazy_static::lazy_static;
use std::time::Duration;
use hhmmss::Hhmmss;

use crate::actions::*;
use crate::transformation::{Transformation, TransformBlockCipher, BlockCipherType, BlockCipherMode, BlockCipherPadding};
use crate::datatype::H2Type;
use crate::datatype::simple::{H2Number, Rgb};
use crate::datatype::composite::H2Struct;
use crate::datatype::composite::string::{H2String, LPString};
use crate::generic_number::{GenericNumber, GenericReader, Endian, BetterEnumFormatter, DefaultFormatter, BooleanFormatter, BitmaskFormatter};

mod helpers;
use helpers::*;

const LAYER: &'static str = "default";

const TERRARIA_KEY: &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";
const TERRARIA_IV:  &[u8] = b"h\x003\x00y\x00_\x00g\x00U\x00y\x00Z\x00";

const OFFSET_SPAWN_POINTS: usize = 0x99c;
const OFFSET_JOURNEY_DATA: usize = 0x6b;

// Things left:
// Cellphone HUDs

#[derive(Debug, Clone, Copy)]
struct TerrariaOffsets {
    magic:          usize,
    name:           usize,

    // Relative to name
    time_played:    usize,
    face:           usize,
    visibility:     usize,
    clothing:       usize,
    health:         usize,
    mana:           usize,
    game_mode:      usize,
    colours:        usize,
    equipment:      usize,
    inventory:      usize,
    coins_and_ammo: usize,
    other_equipment: usize,
    piggy_bank:     usize,
    safe:           usize,
    spawnpoints:    usize,
    buffs:          Option<usize>,
    defenders_forge: usize,
    void_vault:     Option<usize>,

    // Relative to end of spawnpoints
    journey_data:   Option<usize>,
}

lazy_static! {
    /// Offsets for Terraria address from pre-1.4
    static ref TERRARIA_OLD_OFFSETS: TerrariaOffsets = {
        TerrariaOffsets {
            magic:          0x04,
            name:           0x18,

            // Offset from end of name
            game_mode:      0x00,
            time_played:    0x01,
            face:           0x09,
            visibility:     0x0e,
            clothing:       0x11,
            health:         0x12,
            mana:           0x1a,
            colours:        0x28,
            equipment:      0x3d,
            inventory:      0xd3,
            coins_and_ammo: 0x2c7,
            other_equipment: 0x317,
            piggy_bank:     0x349,
            safe:           0x4b1,
            defenders_forge: 0x619,
            void_vault:     None,
            buffs:          None,
            spawnpoints:    0x831,

            // No JourneyMode
            journey_data:   None,
        }
    };

    /// Offsets for Terraria fields from version 1.4 (and up, so far)
    static ref TERRARIA_NEW_OFFSETS: TerrariaOffsets = {
        TerrariaOffsets {
            // Offset from start of file
            magic:           0x04,
            name:            0x18,

            // Offset from end of name
            game_mode:       0x00,
            time_played:     0x01,
            face:            0x09,
            visibility:      0x0e,
            clothing:        0x11,
            health:          0x12,
            mana:            0x1a,
            colours:         0x2a,
            equipment:       0x3f,
            inventory:       0xd5,
            coins_and_ammo:  0x2c9,
            other_equipment: 0x319,
            piggy_bank:      0x34b,
            safe:            0x4b3,
            defenders_forge: 0x61b,
            void_vault:      Some(0x783),
            buffs:           Some(0x8ec),
            spawnpoints:     0x99c,

            // Offset from the end of spawnpoints
            journey_data:   Some(0x6b),
        }
    };

    /// This transformation will decrypt the Terraria savefile
    static ref TRANSFORMATION_DECRYPT: Transformation = {
        TransformBlockCipher::new(
            BlockCipherType::AES,
            BlockCipherMode::CBC,
            BlockCipherPadding::Pkcs7,
            TERRARIA_KEY.to_vec(),
            Some(TERRARIA_IV.to_vec()),
        ).unwrap()
    };

    /// Terraria strings are ASCII, prefixed with a U8 length
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
            ("id".to_string(),          H2Number::new(GenericReader::U32(Endian::Little), BetterEnumFormatter::new("TerrariaItem").unwrap())),
            ("quantity".to_string(),    H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
            ("affix".to_string(),       H2Number::new(GenericReader::U8, BetterEnumFormatter::new("TerrariaAffix").unwrap())),
            ("is_favorite".to_string(), H2Number::new(GenericReader::U8, BooleanFormatter::new())),
        ]).unwrap()
    };

    static ref STORED_ITEM: H2Type = {
        H2Struct::new(vec![
            ("id".to_string(),          H2Number::new(GenericReader::U32(Endian::Little), BetterEnumFormatter::new("TerrariaItem").unwrap())),
            ("quantity".to_string(),    H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
            ("affix".to_string(),       H2Number::new(GenericReader::U8, BetterEnumFormatter::new("TerrariaAffix").unwrap())),
        ]).unwrap()
    };

    static ref EQUIPPED_ITEM: H2Type = {
        H2Struct::new(vec![
            ("id".to_string(),          H2Number::new(GenericReader::U32(Endian::Little), BetterEnumFormatter::new("TerrariaItem").unwrap())),
            ("affix".to_string(),       H2Number::new(GenericReader::U8, BetterEnumFormatter::new("TerrariaAffix").unwrap())),
        ]).unwrap()
    };

    static ref BUFF: H2Type = {
        H2Struct::new(vec![
            ("id".to_string(),          H2Number::new(GenericReader::U32(Endian::Little), BetterEnumFormatter::new("TerrariaBuff").unwrap())),
            ("duration".to_string(),    H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
        ]).unwrap()
    };

    static ref HEALTH_MANA: H2Type = {
        H2Struct::new(vec![
            ("current".to_string(), H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
            ("max".to_string(),     H2Number::new(GenericReader::U32(Endian::Little), DefaultFormatter::new())),
        ]).unwrap()
    };

    static ref COLOURS: H2Type = {
        H2Struct::new(vec![
            ("hair".to_string(),       Rgb::new(true)),
            ("skin".to_string(),       Rgb::new(true)),
            ("eyes".to_string(),       Rgb::new(true)),
            ("shirt".to_string(),      Rgb::new(true)),
            ("undershirt".to_string(), Rgb::new(true)),
            ("pants".to_string(),      Rgb::new(true)),
            ("shoes".to_string(),      Rgb::new(true)),
        ]).unwrap()
    };
}


fn transform_decrypt(record: &mut Record<Action>, buffer: &str) -> SimpleResult<()> {
    record.apply(ActionBufferTransform::new(buffer, *TRANSFORMATION_DECRYPT))
}

/// Special parser for time_played that calculates the proper duration
fn parse_time_played(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    let time_played = create_entry_u64( record, buffer, LAYER, &H2Number::new(GenericReader::U64(Endian::Little), DefaultFormatter::new()), offset, None)?;

    let duration = Duration::from_micros(time_played / 10);
    add_comment(record, buffer, LAYER, offset, &format!("Playtime: {}", duration.hhmmssxxx()))?;

    Ok(())
}

fn parse_visibility(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    create_entry(
        record,
        buffer,
        LAYER,
        &H2Number::new(GenericReader::U16(Endian::Little), BitmaskFormatter::new("TerrariaVisibility", false)?),
        offset, // Offset
        Some("Equipment visibility"),
    )?;

    Ok(())
}

fn parse_equipment(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    add_comment(record, buffer, LAYER, offset,  "Start offset for equipment")?;
    add_comment(record, buffer, LAYER, offset + (10 * 5),  "Start offset for vanity")?;
    add_comment(record, buffer, LAYER, offset + (20 * 5),  "Start offset for dyes")?;

    for i in 0..30 {
        create_entry(
            record,
            buffer,
            LAYER,
            &*EQUIPPED_ITEM,
            offset + (i * 5),
            None,
        )?;
    }

    add_comment(record, buffer, LAYER, offset + (30 * 5) - 1,  "End offset for equipment")?;

    Ok(())
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

fn parse_other_equipment(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    add_comment(record, buffer, LAYER, offset, "Start offset for other equipment")?;

    for i in 0..10 {
        create_entry(
            record,
            buffer,
            LAYER,
            &*EQUIPPED_ITEM,
            offset + (i * 5),
            None,
        )?;
    }

    add_comment(record, buffer, LAYER, offset + 50 - 1, "End offset for other equipment")?;

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

fn parse_defenders_forge(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    add_comment(record, buffer, LAYER, offset,  "Start offset for defender's forge")?;
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
    add_comment(record, buffer, LAYER, offset + 360 - 1, "End offset for defender's forge")?;

    Ok(())
}

fn parse_void_vault(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    add_comment(record, buffer, LAYER, offset,  "Start offset for void vault")?;
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
    add_comment(record, buffer, LAYER, offset + 360 - 1, "End offset for void vault")?;

    Ok(())
}

fn parse_buffs(record: &mut Record<Action>, buffer: &str, offset: usize) -> SimpleResult<()> {
    add_comment(record, buffer, LAYER, offset,  "Start offset for buffs")?;
    // Technically this is an array, but we don't really handle arrays well enough to use one
    for i in (offset..(offset + 176)).step_by(8) {
        create_entry(
            record,
            buffer,
            LAYER,
            &*BUFF,
            i,
            None,
        )?;
    }
    add_comment(record, buffer, LAYER, offset + 176 - 1, "End offset for buffs")?;

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
    let version_number = create_entry_u64(record, buffer, LAYER, &H2Number::new(GenericReader::U32(Endian::Little), BetterEnumFormatter::new("TerrariaVersion")?), 0x00, Some("Version number"))?;

    // Get the offsets for later
    let offsets = if version_number < 230 {
        *TERRARIA_OLD_OFFSETS
    } else {
        *TERRARIA_NEW_OFFSETS
    };

    // Get the "magic" value
    create_entry(record, buffer, LAYER, &H2String::new(7, H2Number::new(GenericReader::ASCII, DefaultFormatter::new()))?, offsets.magic, Some("\"Magic\" value"))?;

    // Create an entry for the name
    let name = create_entry(record, buffer, LAYER, &*TERRARIA_LPSTRING, offsets.name, Some("Character name"))?;

    // The end of the name is the starting offset for the next bunch of fields
    let base = name.actual_range.end as usize;

    // Time played has a special parser because it's a duration value that we
    // want to display pretty
    parse_time_played(record, buffer, base + offsets.time_played)?;

    // Character face is an 8-bit number that we can't erally do much with
    create_entry(record, buffer, LAYER, &H2Number::new(GenericReader::U8, DefaultFormatter::new()), base + offsets.face, Some("Character face"))?;

    // Equipment visibility is a 10-bit bitmask that we've created a definition for
    create_entry(record, buffer, LAYER, &H2Number::new(GenericReader::U16(Endian::Little), BitmaskFormatter::new("TerrariaVisibility", false)?), base + offsets.visibility, Some("Equipment visibility"))?;

    // Clothing is an enumeration (this also includes gender, and oddly enough
    // it's not in the same order as the UI shows)
    create_entry(record, buffer, LAYER, &H2Number::new(GenericReader::U8, BetterEnumFormatter::new("TerrariaClothing").unwrap()), base + offsets.clothing, Some("Character clothing"))?;

    // Health and mana are both a simple struct with current + max
    create_entry(record, buffer, LAYER, &*HEALTH_MANA, base + offsets.health, Some("Health"))?;
    create_entry(record, buffer, LAYER, &*HEALTH_MANA, base + offsets.mana, Some("Mana"))?;

    // Create an entry for the game mode - we'll need this later to determine
    // if we have Journey Mode data
    let game_mode = create_entry_u64(record, buffer, LAYER, &H2Number::new(GenericReader::U8, BetterEnumFormatter::new("TerrariaGameMode")?), base + offsets.game_mode, Some("Game mode"))?;

    // Parse character colours
    create_entry(record, buffer, LAYER, &*COLOURS, base + offsets.colours, Some("Colours"))?;

    // These are all effectively arrays
    parse_equipment(record, buffer, base + offsets.equipment)?;
    parse_inventory(record, buffer, base + offsets.inventory)?;
    parse_coins_and_ammo(record, buffer, base + offsets.coins_and_ammo)?;
    parse_other_equipment(record, buffer, base + offsets.other_equipment)?;
    parse_piggy_bank(record, buffer, base + offsets.piggy_bank)?;
    parse_safe(record, buffer, base + offsets.safe)?;
    parse_defenders_forge(record, buffer, base + offsets.defenders_forge)?;

    if let Some(offset_void_vault) = offsets.void_vault {
        parse_void_vault(record, buffer, base + offset_void_vault)?;
    }

    // Buffs are an option field - 1.4+ only
    if let Some(offset_buffs) = offsets.buffs {
        parse_buffs(record, buffer, base + offset_buffs)?;
    }

    // Spawnpoints consists of zero or more entries, each of which is a fixed
    // length.
    // Everything after spawnpoints (mostly just journeymode data) is relative
    // to the end of spawnpoints
    let new_base = parse_spawnpoints(record, buffer, base + offsets.spawnpoints)?;

    // game_mode 3 == Journey Mode
    if game_mode == 3 {
        // Only parse this if we have a journey_data offset (1.4+)
        if let Some(offset) = offsets.journey_data {
            parse_journeymode(record, buffer, new_base + offset)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::PathBuf;

    use crate::project::H2Project;
    use crate::actions::ActionBufferCreateFromBytes;

    /// All we really do to test is make sure it analyzes cleanly (no errors)
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

        Ok(())
    }
}
