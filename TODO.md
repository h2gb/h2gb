* To-do soon / unsorted
  * Tests and documentation
  * Kill the "static" offset thing, and simply all the things
  * Change structs (and other APIey things) to take &str instead of String
  * Bi-directional enums (so we can fetch stuff by name)
  * Automatically create a default "Layer"
  * Think of some place for "notes" - eg, if a transformation has a comment or warning or something, it should have a place to be seen
  * I don't think I want H2String and such to take a H2Type parameter anymore, instead I'd like a Reader
    * My logic is, we don't need the "display" parameter, so why do we need the full type? A Reader is more logical
  * Rename 'definition' to 'reader' among h2type definitions

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
