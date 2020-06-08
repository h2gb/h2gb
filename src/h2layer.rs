use serde::{Serialize, Deserialize};
use bumpy_vector::BumpyVector;

use crate::h2entry::H2Entry;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Layer {
    name: String,
    show_undefined: bool,
    entries: BumpyVector<H2Entry>,
    //combinators: BumpyVector<H2Combinator>,
}
