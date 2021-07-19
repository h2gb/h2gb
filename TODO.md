* Add some way to 'peek' rather than actually creating an entry
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

Entry create + insert...
* This is the underpinning of the analyzer, and I don't really like it. I don't
  know what'd be better, but I need to think of something
* We need to consider comments, references, create / insert if something changes, etc.
* We also need to consider "peeking" at entries, just to get info about them
* Idea: Re-create the entry when inserting it (inefficient and bad)
* Idea: Code the buffer name into the entry
* Idea: Deal in resolved entries instead of abstract entries?



Responsibilities...

* Project

* Buffer
** Transform / Untransform
** Add / remove layer
** Stuff that uses the underlying data
*** Peek (H2Type -> Resolvedtype)
*** Create an entry in a layer
*** Edit

* Layer

* Entry
