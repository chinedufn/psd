psd [![Build Status](https://travis-ci.org/chinedufn/psd.svg?branch=master)](https://travis-ci.org/chinedufn/psd) [![docs](https://docs.rs/psd/badge.svg)](https://docs.rs/psd)
===============

> A Rust API for parsing and working with PSD files.

## Background / Initial Motivation

I had an `imagemagick` script for a couple of years for exporting specific layers from PSDs but that broken down when I upgraded `imagemagick`.

After a bit of Googling I couldn't land on a solution for my problem so I made this crate.

My approach is to add functionality as I need it so there might be things that you want to do that are currently unsupported.

That said, if there's anything missing that you need please feel very free to open an issue!

## [API Docs](https://docs.rs/psd)

The [API documentation](https://docs.rs/psd).

## Usage

```rust
use psd::{ColorMode, Psd};

fn main () {
    // .. Get a byte slice of PSD file data somehow ..
    let psd = include_bytes!("./my-psd-file.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.color_mode(), ColorMode::Rgb);

    assert_eq!(psd.width(), 500);
    assert_eq!(psd.height(), 500);

    for (layer_name, layer) in psd.layers().iter() {
    }
}
```

## See Also

- [PSD specification](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/) - the basis of our API

## License

MIT
