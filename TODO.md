* Important things
  * Print out structs better (like, each field gets its own thing)
  * Clean up error handling on analyzer (in particular, when you read a bad location)
  * Write tests for character types in GenericNumber - and basically write tests everywhere
  * Write documentation everywhere
  * Figure out Enums better
  * Get rid of the weird "static" offsets in H2Type, I don't think it's buying anything except complication
  * Eliminate u64 as much as possible, in favour of usize
  * BetterEnum -> Enum
  * Maybe kill Union?

* Datatypes
  * Do a better "duration" type
  * Do a timestamp type
  * Bitmap / flags field type

* Analyzers
  * Think of some way to "diff" two binary files
  * Think of a way to make analyzing smoother, like automatically doing offsets and stuff
  * Bitmap
  * .MPQ

* Major features
  * Cross references
  * Named addresses

* Nice-to-haves
  * On entry creation, can we have a function that is "create copies of this type until we reach a specific terminator"? We're running into that a bunch in Terraria analyzer

* Probably pipedreams
  * Can we somehow do comments on struct fields? :-/

* Cleanup
  * Can struct definitions take &str instead of String (for simplicity)

* Things I forget why they're here
  * Move around modules in transformation/ so the transforms are in transform/
