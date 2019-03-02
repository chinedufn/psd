# Major Sections

You can think of a Photoshop file as a byte slice `&[u8]`.

These bytes are organized into 5 major sections, each of which have their own sub-sections.

We represent this in our code using the `MajorSections` type.

```rust
// Imported into the book from `src/sections/mod.rs`

{{#include ../../../../../src/sections/mod.rs:11:35}}
```

Our parsing comes down to reading through the bytes in this byte slice and
using them to create these five major sections.
