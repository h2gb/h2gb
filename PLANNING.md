Interesting usecases:
* .bmp - Large amount of data, odd fields (pixel)
* .png - Compressed data
* .pcap - Multiple sub-messages
* .zip - Encrypted + compressed data
* .tar - A container format with sub-files
* .gz - A compressed format
* .tar.gz - Nested container inside compression
* .exe - A container format with a bunch of different sub-formats

# Goals

We will:

* Analyze arbitrary file formats
* Analyze layered file formats
* Allow automatic annotation
* Allow manual annotation
* Allow editing / exporting updated files (within reason)
* Allow concurrent editing
* Binary diffing? Could probably (dumbly) compare two buffers, tag stuff in a layer
* File dissection?

We will not:

* Convert one format to another
* Do in-depth assembly analysis (we're not IDA/etc)
* Integrate with a debugger

# Overall structure

There are two major touchpoints:

1. Data and actions that are part of the core project, such as buffers, entries,
   undo, redo, etc.

2. Composite actions that take multiple actions on `h2project`, such as an
   analyzer, UI, etc.

## Important questions

How does a binary get loaded / analyzed?

How do references / cross-references work?

How do you edit data?

How does export work?

How do types work?

## Overview?

To load a binary, we load data as an array of u8 into a "buffer".

That "buffer" has layers, and each "layer" has entries.



## h2project

An `h2project` is sort of the core of a working project. It stores the data,
can be saved to disk, and implements all transformations that can change the
data. Holds the undo / redo stack.

Interface:

* H2Project
  * Set metadata
  * Undo
  * Redo
  * Clear undo history
  * Save
  * Export data
  * Get updates since (revision)
* H2Layer
  * Create layer
* H2Entry
  * Create single entry
  * Create multiple entries

## h2buffer

An `h2buffer` contains bytes with a unique name. Data can be imported into an
`h2buffer`, and an `h2buffer` can be exported back to data.

In general, we'll start by importing a file into an `h2buffer`, then analyzing
it. As such, there's a default buffer that we started with - maybe that'll have
a unique name?

An `h2buffer` can become another `h2buffer` in several ways:

* An `h2buffer` can be cloned - cloned buffers bring their data, but nothing
  else
* An `h2buffer` can be extracted - a sequence of bytes in one `h2buffer` can
  become another `h2buffer`; the parent keeps track of its children and vice
  versa (so the parent buffer can be exported, including changes to children)
* An `h2buffer` can be split into a sequence of buffers; each buffer keeps
  track of the buffer before and after itself (again, for exporting)

An `h2buffer` can contain named references to other buffers. In the context of
a PE, that might be different sections - the `.data` buffer knows about the
`.reloc` buffer, say. In the context of a PCAP, it might know the "next" and
"previous" packets. They'll probably also know their parent and children this
way - names like 'parent' and 'next' can have semantic meaning.

A buffer split from another buffer will lose cross references, layers, entries,
and pretty much any information other than the actual bytes. Analyzers will
need to ensure they create buffers before creating references between them.

An empty buffer can be transformed using an `h2transformation`, potentially
multiple times, which changes the underlying data. An `h2buffer` will need to
keep track of which transformations were applied. When possible, a
transformation is two-way (as long as the resulting data is the same length as
the original data).

An `h2buffer` contains:

* A name
* A base (virtual) address
* A block of memory (including length)
* A list of its child buffers
* The name + offset of its parent buffer (if applicable)
* A list of other `h2buffer`s it's linked to (semantic names like `next` / `previous`, or just arbitrary names like `.data` / `.text`)
* Original value for each byte that's been hand-edited (maybe?)
* Whether it's edit-able (if any parent had a one-way transformation, it won't be)
* A list of `h2transformation`s that were applied to it
* A list of `h2layer`s, which contain `h2entry`s
* A list of XReferences to its addresses, possibly from other `h2buffer`s

Actions (performed via the `h2project`):

* Create buffer from data
* Create blank buffer
* Create xref
* Clone / split / extract
* Export
* Edit bytes
* Transform
* More difficult stuff:
  * Rename
  * Change base address
  * Edit

## h2layer

An `h2layer` is the actual `BumpyVector` that holds entries. It's created as a
`MultiVector` entry with a meaningful name (probably like `<buffer>.<layer>` or
something).

Each `h2layer` has an associated `h2buffer`. Every layer of the same buffer is
the same size and has the same base data.

Each `h2entry` is associated with a `h2layer`.

References can be made from any address in any `h2layer`, to any address in
any `h2buffer`. Cross references will be tracked by the `h2buffer`. That way,
every layer will have access to every cross reference.

Actions (performed via the `h2project`):

* Create
* Create reference

## h2entry

A meaningfully defined sequence of bytes within a `h2buffer`, defined and
stored within an `h2layer`. This is the core "thing" users will see.

`h2entry`s within a single layer cannot overlap, but entries in different
layers in the same `h2buffer` they can. They also don't need to be contiguous.

Each `h2entry` contains:

* A starting address and size
* A slice of bytes that represent it, from the `h2buffer`
  * (An entry may or may not know its own bytes.. I'm learning towards not)
* References from it to other offsets (potentially in different `h2buffer`s)
* A user-readable display
* How it was created, and optionally how it can be recreated, which can be:
  * User-defined - nothing special, a user created it by hand, probably gets undefined by edits
  * `h2simpletype` - a simple fixed-length datatype (char, int, etc)
  * `h2complextype` - a complex, variable-length data type
    * These are harder, because we can't re-run them without potentially overlapping with other entries

# Datatypes

Within a layer, we can define certain sequences as a "type". The reason we have
to store datatype information is because we want to re-apply them when editing.

A few characteristics of all datatypes:

* They are a fixed length
* They cannot overlap within a layer
* They are composible, using an `h2combinator`, but are still fixed length

## h2datatype / h2combinator / h2pointer

...

# Ways to manipulate buffers

## h2transformation

An `h2buffer` with no `h2layer`s or `h2entry`s can be transformed in any way
imaginable, which changes (and even resizes!) the underlying data.

When possible, this should be a two-way transformation so data can be exported
back to the original size + value. But it's very important that if you
transform, edit, and de-transform a buffer it winds up being the same length (I
don't think there's a way around that). If doing that would change size - for
example, compression - then it's one-way.

Like many things, this will need to be instantiated and stored along with the
buffer that it transformed.

We *might* consider culling the undo buffer when this is done, to avoid storing
a ton of extraneous data.

Some examples:

* unbase64 (2-way)
* unhex (2-way)
* uncompress (1-way)
* decrypt (maybe 2-way?)
* XOR by a constant (2-way)

I think it'll be important to "detect" which transformations can work. For
example, detect a base64 or hex string and suggest that transformation.
