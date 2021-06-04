use simple_error::SimpleResult;
use crate::sized_number::GenericNumber;

mod binary_options;
pub use binary_options::*;

mod decimal_options;
pub use decimal_options::*;

mod enum_options;
pub use enum_options::*;

mod hex_options;
pub use hex_options::*;

mod octal_options;
pub use octal_options::*;

mod scientific_options;
pub use scientific_options::*;

pub trait SizedOptions {
    fn to_string(&self, number: GenericNumber) -> SimpleResult<String>;
}
