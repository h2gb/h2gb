use std::collections::HashMap;
use lazy_static::lazy_static;
use maplit::hashmap;

lazy_static! {
    pub static ref TERRARIA_VERSION: HashMap<u64, &'static str> = {
        hashmap!{
            230 => "Version1_4_0_1 = 230",
            234 => "Version1_4_1_2 = 234",
            235 => "Version1_4_2   = 235",
            236 => "Version1_4_2_1 = 236",
            237 => "Version1_4_2_2 = 237",
            238 => "Version1_4_2_3 = 238",
        }
    };
}
