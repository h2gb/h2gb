* To-do soon / unsorted
  * Automatically create a default "Layer"
  * Think of some place for "notes" - eg, if a transformation has a comment or warning or something, it should have a place to be seen
  * Look at doing a "nicer" format for types - ie, a C-style struct definition, or something that elides most of the complexity that the current "exported" types have

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
  * Consider something to parse files "linearly", the way "fq" does
  * Maybe consume() methods in Context would be helpful?

* Datatypes
  * Do a better "duration" type
    * Display or datatype?
  * Do a timestamp type
    * Display or datatype?

* Analyzers
  * Think of some way to "diff" two binary files
    * https://github.com/8051Enthusiast/biodiff ?
  * Bitmap image
  * .MPQ
  * Look at the formats in https://github.com/wader/fq/

* Major features
  * Cross references
  * Named addresses
  * Parse log / error log for analyzers
  * UI
  * API

* Things I forget why they're here
  * Move around modules in transformation/ so the transforms are in transform/

* Change bitmasks to allow or require the AND values, not numbered values
  * I found it confusing to add a new bitmask entry
  * Also, some stuff isn't so simple - https://docs.microsoft.com/en-us/windows/win32/debug/pe-format#section-flags
