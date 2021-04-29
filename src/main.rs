use libh2gb::*;

use redo::Record;
use simple_error::SimpleResult;

use crate::actions::*;
use std::fs;
use std::path::PathBuf;
use crate::project::h2project::H2Project;
use crate::analyzer::analyze_terraria;

fn main() -> SimpleResult<()> {
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

    for _ in 1..10000 {
        analyze_terraria(&mut record, "buffer")?;
        record.undo()?;
        record.undo()?;
        record.undo()?;
        record.undo()?;
        record.undo()?;
    }

    analyze_terraria(&mut record, "buffer")?;
    //println!("{}", record.target());

    Ok(())
}
