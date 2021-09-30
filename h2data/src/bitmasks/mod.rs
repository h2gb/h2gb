use std::collections::HashMap;
use simple_error::{SimpleResult, SimpleError, bail};
use lazy_static::lazy_static;

/// Load a Bitmask from a .csv file.
///
/// This requires the CSV to be a string file containing exactly two columns:
/// a numeric column (unique, unsigned values no larger than 63) and a string
/// column representing the "name" of the field.
///
/// The numeric column must be unique, but not every bit between 0 and 63 need
/// be represented.
fn load_from_csv(data: &str) -> SimpleResult<HashMap<u64, String>> {
    let mut out = HashMap::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(data.as_bytes());

    for result in rdr.records() {
        let record = result.map_err(|e| {
            SimpleError::new(format!("Couldn't read bitmask CSV: {}", e))
        })?;

        if record.len() != 2 {
            bail!("Bad bitmask CSV: must be 2 records per line, this line was {}", record.len());
        }

        let number: u64 = record.get(0).ok_or(
            SimpleError::new("Error reading the CSV file")
        )?.parse().map_err(|e| {
            SimpleError::new(format!("Couldn't parse first CSV field as integer: {}", e))
        })?;

        if number > 63 {
            bail!("Bitmask value is impossibly high: {} (mask is 63)", number);
        }

        out.insert(number, record.get(1).ok_or(
            SimpleError::new("Couldn't parse the CSV")
        )?.to_string());
    }

    Ok(out)
}

lazy_static! {
    /// Pre-load the BITMASKS structure
    pub static ref BITMASKS: HashMap<String, HashMap<u64, String>> = {
        let mut h = HashMap::new();
        h.insert("TerrariaVisibility".to_string(), load_from_csv(include_str!("./terraria_visibility.csv")).unwrap());

        h
    };
}

/// Does a bitmask with the given name exist?
pub fn bitmask_exists(name: &str) -> bool {
    BITMASKS.contains_key(name)
}

/// Convert from a 64-bit value to a list of booleans and bits.
///
/// The return type is a list of tuples. The tuples are composed of three
/// values: `(bool is_set, u64 bit_mask, String name)`.
///
/// * `is_set` - true if the value is on, false if it is off
/// * `bit_mask` - the shifted integer value - bit `3` would be `(1<<3)` or `0x08`, for example
/// * `name` - the name of the field, directly from the original CSV file
pub fn from_bitmask(bitmask: &str, value: u64) -> SimpleResult<Vec<(bool, u64, String)>> {
    let bitmask = BITMASKS.get(bitmask).ok_or(
        SimpleError::new(format!("No such bitmask: {}", bitmask))
    )?;

    let mut out = Vec::new();

    for bit in 0..64 {
        // If it doesn't exist...
        let shifted_bit = 1 << bit;
        let is_on = (shifted_bit & value) == shifted_bit;

        // Include it if it's "on" and/or defined
        if is_on || bitmask.contains_key(&bit) {
            //let default_name = format!("Unknown_0x{:x}", 1u64 << bit);
            out.push((is_on, shifted_bit, bitmask.get(&bit).unwrap_or(&format!("Unknown_0x{:x}", shifted_bit)).to_string()));
        }
    }

    Ok(out)
}

/// Convert from a 64-bit value to a list of strings.
///
/// "Negative" strings are only included if `include_negatives` is set.
pub fn from_bitmask_str(bitmask: &str, value: u64, include_negatives: bool) -> SimpleResult<Vec<String>> {
    let mut out: Vec<_> = from_bitmask(bitmask, value)?.iter()
        .filter(|(is_set, _, _)| include_negatives || *is_set)
        .map(|(is_set, _, name)| {
            match *is_set {
                true  => name.to_string(),
                false => format!("~{}", name),
            }
        })
        .collect();

    if out.len() == 0 {
        out.push("(n/a)".to_string());
    }

    Ok(out)
}
