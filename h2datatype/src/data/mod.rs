use std::collections::HashMap;

use simple_error::SimpleResult;

mod constants;
use constants::*;

mod enums;
use enums::*;

mod bitmasks;
use bitmasks::*;

struct DataNg {
    // constants: HashMap<String, Constants>,
    //enums: HashMap<String, Enum>,
    //bitmasks: HashMap<String, Bitmask>,
}

impl DataNg {
    fn load_constants_from_directory(&mut self, directory_name: &str) -> SimpleResult<()> {
        Ok(())
    }
}
