use std::collections::HashMap;
use lazy_static::lazy_static;

mod enums;
use enums::terraria_game_mode::*;
use enums::terraria_buff::*;
use enums::terraria_item::*;
use enums::terraria_version::*;

lazy_static! {
    pub static ref H2ENUMS: HashMap<String, &'static HashMap<u64, &'static str>> = {
        let mut h = HashMap::new();

        h.insert("TerrariaGameMode".to_string(), &*TERRARIA_GAME_MODE);
        h.insert("TerrariaBuff".to_string(),     &*TERRARIA_BUFF);
        h.insert("TerrariaItem".to_string(),     &*TERRARIA_ITEM);
        h.insert("TerrariaVersion".to_string(),  &*TERRARIA_VERSION);

        // h.insert("TestEnum".to_string(),         &*TEST_ENUM);

        // h.insert("WindowsError".to_string(),     &*WINDOWS_ERROR);

        h
    };
}
