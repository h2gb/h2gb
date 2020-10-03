// use serde::{Serialize, Deserialize};
pub mod number;

pub type H2Context<'a> = std::io::Cursor<&'a Vec<u8>>;
