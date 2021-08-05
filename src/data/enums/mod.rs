use std::collections::HashMap;
use simple_error::{SimpleResult, SimpleError, bail};
use lazy_static::lazy_static;

fn load_from_csv(data: &str) -> SimpleResult<HashMap<u64, String>> {
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

        let number: u64 = record.get(0).ok_or(
            SimpleError::new("Error reading the CSV file")
        )?.parse().map_err(|e| {
            SimpleError::new(format!("Couldn't parse first CSV field as integer: {}", e))
        })?;

        out.insert(number, record.get(1).ok_or(
            SimpleError::new("Couldn't parse the CSV")
        )?.to_string());
    }

    Ok(out)
}

lazy_static! {
    /// Enumerations comment
    pub static ref ENUMS: HashMap<String, HashMap<u64, String>> = {
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
