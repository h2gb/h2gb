mod h2entry;
mod h2buffer;
mod h2value;

use h2buffer::H2Buffer;

fn main() {
  let buffer = H2Buffer::new(Box::new([0, 1, 2, 3, 4, 5]));
  println!("{}", buffer);

  /*let entry = H2Entry {
    start: 2,
    length: 3,
    display: String::from("test"),
    data_refs: Box::new([]),
    code_refs: Box::new([]),
  };
  buffer.set(entry);

  buffer.print();
  println!();
  println!("Zero:");
  println!("{}", buffer.get(0).unwrap());*/
}
