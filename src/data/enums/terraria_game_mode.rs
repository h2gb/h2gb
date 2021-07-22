use std::collections::HashMap;
use lazy_static::lazy_static;
use maplit::hashmap;

lazy_static! {
    pub static ref TERRARIA_GAME_MODE: HashMap<u64, &'static str> = {
        hashmap!{
            0 => "Classic",
            1 => "MediumCore",
            2 => "HardCore",
            3 => "JourneyMode",
        }
    };
}
