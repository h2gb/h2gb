* Decisions...
  * Do I do Enums as a datatype or formatter (or both)?
    * Likewise Bitmasks

* To-do soon
  * Tests and documentation
  * Kill Unions - I don't think they're good
  * Investigate nested bumpy vectors
  * Kill the "static" offset thing, and simply all the things
  * Change structs (and other APIey things) to take &str instead of String

* Work on Enum / Bitmask - are they datatypes or output formats?
  * They require external information - from the 'data' package - maybe that's what's different?
  * I need to think about duration and timestamps - they do *not* require external information to display

* Major / annoying cleanup
  * Make bumpyvectors use usize instead of u64

* Nested bumpy vectors
  * Print structs and such more nicely
  * Comments on any field?
  * Make the interface for resolving stricter, to ensure things don't overlap and stuff
    * Guarantee that things are not overlapping
    * Are things required to be contiguous?

* Analyzer improvements
  * Clean up error handling on analyzer (in particular, when you read a bad location)
  * On entry creation, can we have a function that is "create copies of this type until we reach a specific terminator"? We're running into that a bunch in Terraria analyzer
  * An "identify" function for analyzers
    * More importantly, a trait

* Datatypes
  * Do a better "duration" type
    * Display or datatype?
  * Do a timestamp type
    * Display or datatype?

* Analyzers
  * Think of some way to "diff" two binary files
  * Think of a way to make analyzing smoother, like automatically doing offsets and stuff
  * Bitmap image
  * .MPQ

* Major features
  * Cross references
  * Named addresses
  * Parse log / error log for analyzers
  * UI
  * API

* Things I forget why they're here
  * Move around modules in transformation/ so the transforms are in transform/
