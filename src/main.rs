use libh2gb::*;

use std::{env, fs};
use std::path::PathBuf;

use redo::Record;
use simple_error::SimpleResult;

use crate::actions::*;
use crate::project::H2Project;
use crate::analyzer::analyze_terraria;

fn main() -> SimpleResult<()> {
    // Load the data

    let data = fs::read(match env::args().nth(1) {
        Some(f) => {
            PathBuf::from(f)
        },
        None => {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("testdata/terraria/ManySpawnPoints.plr");
            d
        }
    }).unwrap();

    // Create a fresh record
    let mut record: Record<Action> = Record::new(
        H2Project::new("Terraria Test", "1.0")
    );

    // Load the file data into a new buffer
    let action = ActionBufferCreateFromBytes::new("buffer", &data, 0x0);
    record.apply(action)?;

    // for _ in 1..10000 {
    //     analyze_terraria(&mut record, "buffer")?;
    //     record.undo()?;
    //     record.undo()?;
    //     record.undo()?;
    //     record.undo()?;
    //     record.undo()?;
    // }

    match analyze_terraria(&mut record, "buffer") {
        Ok(_) => (),
        Err(e) => println!("Something went wrong: {}", e),
    };

    println!("{}", record.target());

    Ok(())
}
