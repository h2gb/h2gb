use std::collections::HashMap;
use simple_error::{SimpleResult, SimpleError, bail};
use lazy_static::lazy_static;

fn load_from_csv(data: &str) -> SimpleResult<Vec<(u64, String)>> {
    let mut out = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(data.as_bytes());

    for result in rdr.records() {
        let record = result.map_err(|e| {
            SimpleError::new(format!("Couldn't read CSV: {}", e))
        })?;

        if record.len() != 2 {
            bail!("Bad enum CSV: must be 2 records per line, this line was {}", record.len());
        }

        let number: u64 = record.get(0).ok_or(
            SimpleError::new("Error reading the CSV file")
        )?.parse().map_err(|e| {
            SimpleError::new(format!("Couldn't parse first CSV field as integer: {}", e))
        })?;

        out.push((number, record.get(1).ok_or(
            SimpleError::new("Couldn't parse the CSV")
        )?.to_string()));
    }

    Ok(out)
}

lazy_static! {
    pub static ref BITMASKS: HashMap<String, Vec<(u64, String)>> = {
        let mut h = HashMap::new();
        h.insert("TerrariaVisibility".to_string(), load_from_csv(include_str!("./terraria_visibility.csv")).unwrap());

        h
    };
}

pub fn bitmask_exists(name: &str) -> bool {
    BITMASKS.contains_key(name)
}

pub fn from_bitmask(bitmask: &str, mut value: u64) -> SimpleResult<(Vec<(u64, &str, bool)>, u64)> {
    let bitmask = BITMASKS.get(bitmask).ok_or(
        SimpleError::new(format!("No such bitmask: {}", bitmask))
    )?;

    let out: Vec<(u64, &str, bool)> = bitmask.iter().map(|(bit, name)| {
        let line = (value, &name[..], ((1 << bit) & value) != 0);

        value = value & !(1 << bit);

        line
    }).collect();

    Ok((out, value))
}