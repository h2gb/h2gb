use redo::Record;
use simple_error::{bail, SimpleResult};

use h2datatype::data::{Data, LoadOptions, LoadNamespace, LoadName};
use h2datatype::simple::{H2Bitmask, H2Enum, Rgb, H2Blob};
use h2datatype::simple::numeric::H2Integer;
use h2datatype::simple::string::{H2String, LPString};
use h2datatype::composite::H2Struct;

use generic_number::{Integer, IntegerReader, CharacterReader, CharacterFormatter, Endian, DefaultFormatter, BooleanFormatter, HexFormatter};

use crate::actions::*;
use crate::analyzer::helpers::*;

const LAYER: &'static str = "default";

fn analyze_windows_specific_optional_header(record: &mut Record<Action>, buffer: impl AsRef<str>, data: &Data, start: usize, pe_format: &'static str) -> SimpleResult<()> {
    let mut offset = start;

    if pe_format == "PE32" {
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Image base"),
            &data
        )?;
        offset += 4;
    } else {
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U64(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Image base"),
            &data
        )?;
        offset += 8;
    }

    // Section alignment 4
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Section alignment"),
        &data
    )?;
    offset += 4;

    // File alignment 4
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("File alignment"),
        &data
    )?;
    offset += 4;

    // Major OS version 2
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Major OS version"),
        &data
    )?;
    offset += 2;

    // Minor OS version 2
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Minor OS version"),
        &data
    )?;
    offset += 2;

    // Major image version 2
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Major image version"),
        &data
    )?;
    offset += 2;

    // Minor image version 2
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Minor image version"),
        &data
    )?;
    offset += 2;

    // Major subsystem version 2
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Major subsystem version"),
        &data
    )?;
    offset += 2;

    // Minor subsystem version 2
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Minor subsystem version"),
        &data
    )?;
    offset += 2;

    // Win32 version value 4
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Win32 version"),
        &data
    )?;
    offset += 4;

    // Size of image 4
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Size of image"),
        &data
    )?;
    offset += 4;

    // Size of headers 4
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Size of headers"),
        &data
    )?;
    offset += 4;

    // Checksum 4
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Checksum"),
        &data
    )?;
    offset += 4;

    // Subsystem 2
    create_entry(
        record,
        &buffer,
        LAYER,

        H2Enum::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty(), Some("pe"), "subsystems", data)?,

        offset,
        Some("Subsystem"),
        &data
    )?;
    offset += 2;

    // Dll characteristics 2
    create_entry(
        record,
        &buffer,
        LAYER,

        H2Bitmask::new(IntegerReader::U16(Endian::Little), Some(HexFormatter::new_pretty().into()), Some("pe"), "dll_characteristics", false, data)?,

        offset,
        Some("DLL Characteristics"),
        &data
    )?;
    offset += 2;

    if pe_format == "PE32" {
        // Size of stack reserve 4/8
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Size of stack reserve"),
            &data
        )?;
        offset += 4;

        // Size of stack commit 4/8
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Size of stack commit"),
            &data
        )?;
        offset += 4;

        // Size of heap reserve 4/8
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Size of heap reserve"),
            &data
        )?;
        offset += 4;

        // Size of heap commit 4/8
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Size of heap commit"),
            &data
        )?;
        offset += 4;
    } else {
        // Size of stack reserve 4/8
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U64(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Size of stack reserve"),
            &data
        )?;
        offset += 8;

        // Size of stack commit 4/8
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U64(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Size of stack commit"),
            &data
        )?;
        offset += 8;

        // Size of heap reserve 4/8
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U64(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Size of heap reserve"),
            &data
        )?;
        offset += 8;

        // Size of heap commit 4/8
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U64(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Size of heap commit"),
            &data
        )?;
        offset += 8;
    }

    // Loader flags 4 (reserved)
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Loader flags"),
        &data
    )?;
    offset += 4;

    // Number of RVA and sizes 4
    let number_of_rva: usize = create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Number of RVA and sizes"),
        &data
    )?.as_integer.unwrap().try_into().unwrap();
    offset += 4;

    let data_directory_names = vec![
        "Export Table",
        "Import Table",
        "Resource Table",
        "Exception Table",
        "Certificate Table",
        "Base Relocation Table",
        "Debug",
        "Architecture",
        "Global Ptr",
        "TLS Table",
        "Load Config Table",
        "Bound Import",
        "IAT",
        "Delay Import Descriptor",
        "CLR Runtime Header",
        "Reserved",
    ];
    for i in 0..number_of_rva {
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some(format!("Data Directory {} (Address)", data_directory_names[i]).as_ref()),
            &data
        )?;
        offset += 4;

        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some(format!("Data Directory {} (Size)", data_directory_names[i]).as_ref()),
            &data
        )?;
        offset += 4;
    }

    Ok(())
}

fn analyze_optional_header(record: &mut Record<Action>, buffer: impl AsRef<str>, data: &Data, start: usize) -> SimpleResult<()> {
    let mut offset = start;

    let opt_magic: usize = create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Optional header magic"),
        &data,
    )?.as_integer.unwrap().try_into().unwrap();
    offset += 2;

    let pe_format = match opt_magic {
        0x10b => "PE32",
        0x20b => "PE32+",
        _ => bail!("Unknown optional header magic byte: {:x}", opt_magic),
    };

    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U8, HexFormatter::new_pretty()),
        offset,
        Some("Major linker version"),
        &data
    )?;
    offset += 1;

    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U8, HexFormatter::new_pretty()),
        offset,
        Some("Minor linker version"),
        &data
    )?;
    offset += 1;

    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Size of code"),
        &data
    )?;
    offset += 4;

    // Size of initialized data
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Size of initialized data"),
        &data
    )?;
    offset += 4;

    // Size of uninitialized data
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Size of uninitialized data"),
        &data
    )?;
    offset += 4;

    // Address of entrypoint
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Entrypoint (relative to image base)"),
        &data
    )?;
    offset += 4;

    // Base of code
    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Base of code (relative to image base)"),
        &data
    )?;
    offset += 4;

    if pe_format == "PE32" {
        // Optional: Base of data
        create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Base of data"),
            &data
        )?;
        offset += 4;
    }

    analyze_windows_specific_optional_header(record, buffer, data, offset, pe_format)?;

    Ok(())
}

pub fn analyze_relocs(record: &mut Record<Action>, buffer: impl AsRef<str>, data: &Data, start: usize, size: usize) -> SimpleResult<()> {
    let mut offset = start;

    println!("{:?}", size);
    while (offset - start) < size {
        let page_rva: usize = create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Reloc: Page RVA"),
            &data
        )?.as_integer.unwrap().try_into().unwrap();
        offset += 4;

        let block_size: usize = create_entry(
            record,
            &buffer,
            LAYER,
            H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
            offset,
            Some("Reloc: Block size"),
            &data
        )?.as_integer.unwrap().try_into().unwrap();
        offset += 4;

        // Subtract 8 for the size of the header
        for _ in (0..(block_size - 8)).step_by(2) {
            let entry: usize = peek_entry(
                record,
                &buffer,
                &H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()).into(),
                offset,
                data
            )?.as_integer.unwrap().try_into().unwrap();

            let reloc_type = match entry >> 12 {
                0  => "IMAGE_REL_BASED_ABSOLUTE",
                1  => "IMAGE_REL_BASED_LOW",
                2  => "IMAGE_REL_BASED_HIGHLOW",
                3  => "IMAGE_REL_BASED_HIGHADJ",
                4  => "IMAGE_REL_BASED_SPECIAL",
                5  => "IMAGE_REL_BASED_RESERVED",
                6  => "IMAGE_REL_BASED_THUMB_MOV32",
                7  => "IMAGE_REL_BASED_SPECIAL2",
                8  => "IMAGE_REL_BASED_SPECIAL3",
                9  => "IMAGE_REL_BASED_MIPS_JMPADDR16",
                10 => "IMAGE_REL_BASED_DIR64",
                _  => "(unknown)",
            };

            let reloc_offset = entry & 4095;

            create_entry(
                record,
                &buffer,
                LAYER,
                H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
                offset,
                Some(format!("{} @ 0x{:08x}", reloc_type, page_rva + reloc_offset).as_ref()),
                &data
            )?;
            offset += 2;
        }

        // Each block must start on a 32-bit boundary
        if (offset % 2) != 0 {
            offset += 2;
        }
    }

    Ok(())
}

fn analyze_section_header(record: &mut Record<Action>, buffer: impl AsRef<str>, data: &Data, start: usize) -> SimpleResult<()> {
    let mut offset = start;

    let name = create_entry(
        record,
        &buffer,
        LAYER,

        H2String::new(8, CharacterReader::ASCII, CharacterFormatter::new_pretty_str())?,

        offset,
        Some("Name"),
        &data,
    )?.as_string.unwrap();
    offset += 8;

    let virtual_size: usize = create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Virtual size"),
        &data
    )?.as_integer.unwrap().try_into().unwrap();
    offset += 4;

    let virtual_address: usize = create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Virtual address"),
        &data
    )?.as_integer.unwrap().try_into().unwrap();
    offset += 4;

    let raw_size: usize = create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Size of raw data"),
        &data
    )?.as_integer.unwrap().try_into().unwrap();
    offset += 4;

    let raw_data_address: usize = create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Pointer to raw data"),
        &data
    )?.as_integer.unwrap().try_into().unwrap();
    offset += 4;

    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Pointer to relocations"),
        &data
    )?;
    offset += 4;

    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Pointer to line numbers"),
        &data
    )?;
    offset += 4;

    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Number of relocations"),
        &data
    )?;
    offset += 2;

    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Number of line numbers"),
        &data
    )?;
    offset += 2;

    create_entry(
        record,
        &buffer,
        LAYER,
        H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty()),
        offset,
        Some("Characteristics"),
        &data
    )?;
    //offset += 4;

    if raw_size > 0 && raw_data_address > 0 {
        if name == ".reloc\\0\\0" {
            analyze_relocs(record, buffer, data, raw_data_address, virtual_size)?;
        } else {
            create_entry(
                record,
                &buffer,
                LAYER,
                H2Blob::new(raw_size, HexFormatter::new_pretty())?,
                raw_data_address,
                Some(format!("Section: {}", name).as_ref()),
                &data
            )?;
        }
    }


    Ok(())
}

pub fn analyze_pe(record: &mut Record<Action>, buffer: impl AsRef<str>, data: &Data) -> SimpleResult<()> {
    record.apply(ActionLayerCreate::new(&buffer, LAYER))?;

    let pe_offset_type = H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty());
    let pe_offset = create_entry(record, &buffer, LAYER, pe_offset_type, 0x3cusize, Some("PE offset"), &data)?;
    let pe_offset: usize = pe_offset.as_integer.unwrap().try_into().unwrap();

    let pe_magic_type = H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty());
    let pe_magic: u32 = peek_entry(record, &buffer, &pe_magic_type.into(), pe_offset, &data)?.as_integer.unwrap().try_into().unwrap();
    // "PE\0\0"
    if pe_magic != 0x00004550 {
        bail!("Magic PE value not present");
    }

    let pe_magic_type = H2String::new(4, CharacterReader::ASCII, CharacterFormatter::new_pretty_str()).unwrap();
    let pe_magic = create_entry(record, &buffer, LAYER, pe_magic_type, pe_offset, Some("PE Magic"), &data)?;
    if &pe_magic.as_string.unwrap() != "PE\\0\\0" {
        bail!("Magic PE value was not correct");
    }

    let machine_type = H2Enum::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty(), Some("pe"), "machine_types", data)?;
    create_entry(record, &buffer, LAYER, machine_type, pe_offset + 4, Some("Machine"), &data)?;

    let number_of_sections_type = H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty());
    let number_of_sections: usize = create_entry(
        record,
        &buffer,
        LAYER,
        number_of_sections_type,
        pe_offset + 6,
        Some("Number of sections"),
        &data
    )?.as_integer.unwrap().try_into().unwrap();

    let time_date_stamp_type = H2Integer::new(IntegerReader::U32(Endian::Little), DefaultFormatter::new());
    create_entry(record, &buffer, LAYER, time_date_stamp_type, pe_offset + 8, Some("Timestamp"), &data)?;

    let pointer_to_symbol_table_type = H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty());
    create_entry(record, &buffer, LAYER, pointer_to_symbol_table_type, pe_offset + 12, Some("Pointer to symbol table"), &data)?;

    let number_of_symbols_type = H2Integer::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty());
    create_entry(record, &buffer, LAYER, number_of_symbols_type, pe_offset + 16, Some("Number of symbols"), &data)?;

    let size_of_optional_header_type = H2Integer::new(IntegerReader::U16(Endian::Little), HexFormatter::new_pretty());
    let size_of_optional_header: usize = create_entry(record, &buffer, LAYER, size_of_optional_header_type, pe_offset + 20, Some("Size of optional header"), &data)?.as_integer.unwrap().try_into().unwrap();

    let characteristics_type = H2Bitmask::new(IntegerReader::U16(Endian::Little), Some(HexFormatter::new_pretty().into()), Some("pe"), "characteristics", false, data)?;
    create_entry(record, &buffer, LAYER, characteristics_type, pe_offset + 22, Some("Characteristics"), &data)?;

    if size_of_optional_header > 0 {
        analyze_optional_header(record, &buffer, data, pe_offset + 24)?;
    }

    let mut offset = pe_offset + 24 + size_of_optional_header;
    for _ in 0..number_of_sections {
        analyze_section_header(record, &buffer, data, offset)?;
        offset += 40;
    }


    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::PathBuf;

    use h2datatype::data::{LoadOptions, LoadNamespace, LoadName};

    use crate::project::H2Project;
    use crate::actions::ActionBufferCreateFromBytes;

    /// All we really do to test is make sure it analyzes cleanly (no errors)
    #[test]
    fn test_analyze_pe() -> SimpleResult<()> {
        let mut data = Data::default();

        let path = &[env!("CARGO_MANIFEST_DIR"), "testdata/pe/enums"].iter().collect::<PathBuf>();
        data.enums.load_path(&path, &LoadOptions::new(LoadNamespace::Specific("pe".to_string()), LoadName::Auto))?;

        let path = &[env!("CARGO_MANIFEST_DIR"), "testdata/pe/bitmasks"].iter().collect::<PathBuf>();
        data.bitmasks.load_path(&path, &LoadOptions::new(LoadNamespace::Specific("pe".to_string()), LoadName::Auto))?;

        // Load the file_data
        let path = &[env!("CARGO_MANIFEST_DIR"), "../testdata/pe/cmd.exe"].iter().collect::<PathBuf>();
        let file_data = fs::read(path).unwrap();

        // Create a fresh record
        let mut record: Record<Action> = Record::new(
            H2Project::new("PE Test", "1.0")
        );

        let action = ActionBufferCreateFromBytes::new("buffer", &file_data, 0x0);
        record.apply(action)?;

        analyze_pe(&mut record, "buffer", &data)?;

        println!("{}", record.target());

        Ok(())
    }
}
