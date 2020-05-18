Interesting usecases:
* .bmp - Large amount of data, odd fields (pixel)
* .png - Compressed data
* .pcap - Multiple sub-messages
* .zip - Encrypted + compressed data
* .tar - A container format with sub-files
* .gz - A compressed format
* .tar.gz - Nested container inside compression
* .exe - A container format with a bunch of different sub-formats

# Questions

* What's the best way to implement undo / redo?

# Goals

We will:

* Analyze arbitrary file formats
* Analyze layered file formats
* Allow automatic annotation
* Allow manual annotation
* Allow editing / exporting updated files (within reason)
* Allow concurrent editing

We will not:

* Convert one format to another
* Do in-depth assembly analysis (we're not IDA/etc)

# Overall structure

## h2project

An `h2project` is sort of the core of a working project. An instance of the
application creates/loads a single project. It contains one base `h2buffer`
representing the full content of the file, but other `h2buffer`s can be created
as well. It provides an API interface.

This will be the ultimate gateway for all actions, which is what's gonna let us
have undo/redo functions. All actions can be inverted!

Some of what it does:

* Load a new binary, create the base `h2buffer`
* Export back to a binary file
* Save/load to disk
* Undo / redo
* Provide an API interface to perform actions
  * Can be leveraged by: analyzers, web, console, script, etc?

What it contains:

* One or more `h2buffer`s, indexed by name
* A record of all actions performed
* Project metadata - name, version, etc.

The very core is going to be the `h2action`s and `h2query`s, since that will
be the interface to literally everything else.

### h2action

This section lists all actions that can be performed on an `h2project` (or
anything contained therein). Remember that every action must be invertable,
unless we can comfortably cut off the undo buffer when performed.

Simple actions:

* H2Project
  * SetMetadata
    * Name
  * Undo
  * Redo
  * Save
  * Load
  * CreateH2Buffer
  * ExportH2Buffer
* H2Buffer
  * SetMetadata
    * BaseAddress
    * (Setting Name is a bad idea)
  * CreateH2Buffer
  * Transform
  * Clone
  * CreateH2Layer
  * DeleteH2Layer
  * EditBytes
* H2Layer
  * SetMetadata
    * ShowUndefined
    * Name
  * CreateH2Entry
  * DeleteH2Entry
  * UndefineRange
* H2Entry
  * SetMetadata
    * HumanReadable
  * AddReference
  * RemoveReference
* H2Datatype
  * CreateH2Constant
  * CreateH2Enum
  * CreateH2Bitmap

### h2query

Whereas `h2action` does something, an `h2query` retrieves some information.
Anything that will be displayed to the user or used by an analyzer must be done
through this interface.

I'm not sure I'll even need all these, ultimately.

Queries:

* GetEverything
* GetUpdatesSince(rev)
* GetActions
* GetBuffers
* GetLayers
* GetLayersIn(buffer)
* GetEntries
* GetEntriesIn(layer)

## h2buffer

An instance of `h2buffer` contains a bunch of bytes with a unique name. By
default there is a single base `h2buffer` with all data, but more can be
created. It's ultimately identified by a project-unique name.

An `h2buffer` can be created from any sequence of bytes, including a file, a
part of another `h2buffer`, or even a bunch of bytes pasted by the user. 

When an `h2buffer` is split from another buffer, it keeps track of the offset
it came from. A parent keeps a list of its children, as well, so edited data
can be merged back together if the user wants to export to a new binary. If
multiple buffers overlap, that's up to the user to figure out.

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

## h2layer

An `h2layer` is an overlay on top of an `h2buffer` that holds  `h2entry`s. An
`h2buffer` can contain multiple `h2layer`s, and an `h2layer` can contain
multiple non-overlapping `h2entry`s. Different `h2layer`s can have overlapping
`h2entry`s, however.

References can be made from any `h2entry` in any `h2layer` to any address in
any `h2buffer`. Cross references will be tracked by the `h2buffer`. That way,
every layer will have access to every cross reference.

The reasoning for multiple layers: say you're analyzing an executable. You
might do a basic `strings`-type analysis to create an `h2layer` with a bunch of
strings. But then you want to analyze as an executable. That's another layer!

An `h2layer` contains:

* A series of non-overlapping `h2entry`s
* A boolean whether or not to display undefined entries

When data changes (either a user edits it or merges a child buffer's data),
each layer will need to go through its entries to update them.

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
  * `h2combinator` - a complex, fixed-length datatype
    * Since `h2combinator`s define multiple `h2entry`s, it either needs to reference the first field or share a combinator

# `h2datatype`

An `h2datatype` is a class/interface that knows how to take an `h2layer` + an
offset and create (or update) an `h2entry`. Some are simple and built-in, such
as integers and floats - they just need some configuration (like size and
endianness).

Others types are more complex, such as arrays and structs, and can infinitely
nest. These will need to be "configured", in a sense. They will also have
configurations such as padding and alignment.

Yet other types are different types of constants: constants, enums, and
bitmaps. Constants are mappings that, when changed, go away. Enums are mappings
that are grouped, meaning that when the data changes, the new value can be
updated. And bitmaps are bit arrays where each bit's state is meaningful.

The final type, which may or may not make the final cut, are dynamically-sized
types such as strings.

The important part is that an entry knows which datatype created it, so it can
update itself as needed.

**Question:  Do I want these to _create_ `h2entry`s, or do I want h2entrys to reference them and calculate as-needed?**

Ultimately, I'd like to create as many types as possible by parsing header
files.

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

## h2analyzer

These will take an `h2buffer` (not necessarily the base buffer), and perform
any of the operations above to define `h2layer`s, `h2entry`s, and so on. 

I think there will be different types:

* File format analyzers, which know certain formats and can annotate
* Dumb analyzers, which look for and tag certain things (like finding strings)
