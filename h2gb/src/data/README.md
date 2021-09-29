***Note: This file was automatically generated from lib.rs or mod.rs***

A library for loading data that analyzers and users can consume.

Currently, there are two datatypes that we use: Enum and Bitmask. They are
used by the datatypes `H2Enum` and `H2Bitmask` respectively. You probably
don't want to use these directly.

There's also nothing stopping us from loading new Enums or Bitmasks at
runtime. We'll have to see if that's reasonable.

I'd also like to parse other formats besides CSV eventually.

## Enums

An enum is a collection of named values, with a type. For example,
Terraria's "game mode" has 4 possible values (0 = "Normal",
3 = "Journey Mode", etc).

Enums are stored as .csv files that are included at compile-time. The
format is simply `<value>,<description>`. The only restriction is that the
keys must be unique. They must be added to [enums/mod.rs](enums/mod.rs) as
well.

You can see examples of enums in the [enums/](enums/) folder.

## Bitmask

Bitmasks are similar to enums, in that they are loaded from .csv files. The
big difference is that a single bitmask is made up of values between 0 and
63 that each correspond to bits. For example, the following configuration:

```csv
0,VALUE0
1,VALUE1
2,VALUE2
```

means that bit 0 (the rightmost) is `VALUE0`, bit 1 is `VALUE1`, and bit 2
is `VALUE2`. That means if you match the number 0x05 (0101 in binary), it'll
be `VALUE0 | ~VALUE1 | VALUE2`.

License: MIT
