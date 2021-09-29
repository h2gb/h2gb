***Note: This file was automatically generated from lib.rs or mod.rs***

A vector-like object where elements can be larger than one item.

We use this primarily to represent objects in a binary that are made up of
one or more bytes.

## Usage

Instantiate with a maximum size, then use somewhat like a vector:

```rust
use bumpy_vector::{BumpyEntry, BumpyVector};

// Instantiate with a maximum size of 100 and a type of String
let mut v: BumpyVector<String> = BumpyVector::new(100);

// Create a 10-byte entry at the start
let entry: BumpyEntry<String> = BumpyEntry {
  entry: String::from("hello"),
  range: 0..10,
};

// Insert it into the BumpyVector
assert!(v.insert(entry).is_ok());

// Create another entry, this time from a tuple, that overlaps the first
let entry: BumpyEntry<String> = (String::from("error"), 1..6).into();
assert!(v.insert(entry).is_err());

// Create an entry that's off the end of the object
let entry: BumpyEntry<String> = (String::from("error"), 1000..1005).into();
assert!(v.insert(entry).is_err());

// There is still one entry in this vector
assert_eq!(1, v.len());
```

## Serialize / deserialize

Serialization support using [serde](https://serde.rs/) is enabled. The
`BumpyVector` can be serialized with any of the serializers that Serde
supports, such as [ron](https://github.com/ron-rs/ron):

```rust
use bumpy_vector::BumpyVector;

fn main() {
    let mut h: BumpyVector<String> = BumpyVector::new(10);
    h.insert((String::from("a"), 1..3).into()).unwrap();

    // Serialize
    let serialized = ron::ser::to_string(&h).unwrap();

    // Deserialize
    let h: BumpyVector<String> = ron::de::from_str(&serialized).unwrap();
}
```

License: MIT
