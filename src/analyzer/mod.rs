//! Some good info should go here!
use redo::Record;
use simple_error::SimpleResult;

use crate::project::actions::*;
//use crate::project::h2project::H2Project;
use crate::transformation::{TransformBlockCipher, BlockCipherType, BlockCipherMode, BlockCipherPadding};

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
    use crate::project::actions::ActionBufferCreateFromBytes;


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
        let action = ActionBufferCreateFromBytes::new("buffer", &data, 0x80000000);
        record.apply(action)?;

        analyze_terraria(&mut record, "buffer")?;

        println!("{:?}", &record);

        Ok(())
    }
}