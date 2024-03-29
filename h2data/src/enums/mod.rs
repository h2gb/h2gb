use std::collections::HashMap;
use simple_error::{SimpleResult, SimpleError, bail};
use lazy_static::lazy_static;

/// Load an Enum from a .csv file.
///
/// This requires the CSV to be a string file containing exactly two columns:
/// a numeric column (compatible with an unsigned 64-bit value) and a string
/// column representing the "name".
///
/// The numeric column must be unique.
fn load_from_csv(data: &str) -> SimpleResult<HashMap<usize, String>> {
    let mut out = HashMap::new();

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

        let number: usize = record.get(0).ok_or(
            SimpleError::new("Error reading the CSV file")
        )?.parse().map_err(|e| {
            SimpleError::new(format!("Couldn't parse first CSV field as integer: {}", e))
        })?;

        if out.contains_key(&number) {
            bail!("Duplicate key in CSV enum: {}", number);
        }

        out.insert(number, record.get(1).ok_or(
            SimpleError::new("Couldn't parse the CSV")
        )?.to_string());
    }

    Ok(out)
}

lazy_static! {
    /// Enumerations comment
    pub static ref ENUMS: HashMap<String, HashMap<usize, String>> = {
        let mut h = HashMap::new();
        h.insert("TerrariaAffix".to_string(),    load_from_csv(include_str!("./terraria_affix.csv")).unwrap());
        h.insert("TerrariaBuff".to_string(),     load_from_csv(include_str!("./terraria_buff.csv")).unwrap());
        h.insert("TerrariaGameMode".to_string(), load_from_csv(include_str!("./terraria_game_mode.csv")).unwrap());
        h.insert("TerrariaItem".to_string(),     load_from_csv(include_str!("./terraria_item.csv")).unwrap());
        h.insert("TerrariaVersion".to_string(),  load_from_csv(include_str!("./terraria_version.csv")).unwrap());
        h.insert("TerrariaClothing".to_string(), load_from_csv(include_str!("./terraria_clothing.csv")).unwrap());

        h.insert("TestEnum".to_string(),         load_from_csv(include_str!("./test_enum.csv")).unwrap());

        h.insert("WindowsError".to_string(),     load_from_csv(include_str!("./windows_error.csv")).unwrap());

        h
    };
}

pub fn enum_exists(name: &str) -> bool {
    ENUMS.contains_key(name)
}

pub fn from_enum(name: &str, value: usize) -> SimpleResult<Option<&str>> {
    Ok(ENUMS.get(name).ok_or(
        SimpleError::new(format!("No such enum: {}", name))
    )?.get(&value).map(|s| &s[..]))
}
