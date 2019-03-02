# Introduction

`psd` provides a Rust API for parsing and working with the [Adobe Photoshop File Format](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/#50577409_89817).

`psd` seeks to make it easy for you to write scripts that work with Photoshop files.

For example, the original use case that motivated the creationg of the `psd` crate was to be part of a Rust
script that iterated over a directory full of PSD's and combined all of them into a texture atlas (using the
[texture_packer] crate).

---

The Photoshop specification is large so, while we support the main parts of it, not every little bit
is supported.

If there's something that you'd like to get your hands on please feel free to [open an issue].

[texture_packer]: https://github.com/PistonDevelopers/texture_packer
[open an issue]: https://github.com/chinedufn/psd/issues
