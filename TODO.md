* To-do soon / unsorted
  * Tests and documentation
  * Kill the "static" offset thing, and simply all the things
  * Change structs (and other APIey things) to take &str instead of String
  * Bi-directional enums (so we can fetch stuff by name)
  * A "blob" datatype, which is a big U8 array but that isn't displayed
  * Automatically create a default "Layer"
  * Can I move Float out of GenericNumber to make it more useful (sortable / comparable / etc)?
    * Can GenericNumber use more Into() methods instead of as_u64() and stuff like that?
  * Think of some place for "notes" - eg, if a transformation has a comment or warning or something, it should have a place to be seen

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
  * Think of a way to make analyzing smoother, like automatically doing offsets and stuff

* Datatypes
  * Do a better "duration" type
    * Display or datatype?
  * Do a timestamp type
    * Display or datatype?

* Analyzers
  * Think of some way to "diff" two binary files
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
