* Print out structs better (like, each field gets its own thing)
* Can we somehow do comments on struct fields? :-/
* Print unallocated addresses
* Figure out better way to report errors
* Can struct definitions take &str instead of String (for simplicity)
* On entry creation, can we have a function that is "create copies of this type until we reach a specific terminator"? We're running into that a bunch in Terraria analyzer
* Fix the error "failed to fill the whole buffer", which happens if you go off the end of the buffer

* Cross references
* Named addresses

* Move around modules in transformation/ so the transforms are in transform/
* Write tests for character types in GenericNumber
* Eliminate u64 as much as possible, in favour of usize

* Find a new home for Enums that can be parsed
* Get rid of the weird "static" offsets in H2Type, I don't think it's buying anything except complication
* Time values - "this value is a time in seconds" for example
