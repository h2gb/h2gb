pub mod sized_number;

pub type H2Context<'a> = std::io::Cursor<&'a Vec<u8>>;
