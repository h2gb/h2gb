#![allow(dead_code)] // TODO: Disable this

use std::collections::HashMap;

use simple_error::SimpleResult;

mod h2constants;
use h2constants::*;

struct H2DataNg {
    // constants: HashMap<String, H2Constants>,
    //enums: HashMap<String, H2Enum>,
    //bitmasks: HashMap<String, H2Bitmask>,
}

impl H2DataNg {
    fn load_constants_from_directory(&mut self, directory_name: &str) -> SimpleResult<()> {
        Ok(())
    }
}
