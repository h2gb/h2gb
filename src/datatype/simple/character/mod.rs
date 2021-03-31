// Types that don't have subtypes.
//
// Keeping these types together in this module are a convention, there's no
// firm rule.

mod ascii;
pub use ascii::*;

mod utf8;
pub use utf8::*;

mod utf16;
pub use utf16::*;

mod utf32;
pub use utf32::*;

pub mod common;
