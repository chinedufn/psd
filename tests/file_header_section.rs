use anyhow::Result;
use psd::PsdDepth;
use psd::{ColorMode, Psd};

/// cargo test --test file_header_section file_header_section -- --exact
#[test]
fn file_header_section() -> Result<()> {
    let psd = include_bytes!("./fixtures/green-1x1.psd");

    let psd = Psd::from_bytes(psd)?;

    assert_eq!(psd.width(), 1);
    assert_eq!(psd.height(), 1);

    assert_eq!(psd.depth(), PsdDepth::Eight);

    assert_eq!(psd.color_mode(), ColorMode::Rgb);

    Ok(())
}

/// cargo test --test file_header_section negative_top_left -- --exact
#[test]
fn negative_top_left() -> Result<()> {
    let psd = include_bytes!("./fixtures/negative-top-left-layer.psd");

    let psd = Psd::from_bytes(psd)?;

    assert_eq!(psd.width(), 1);
    assert_eq!(psd.height(), 1);

    assert_eq!(psd.depth(), PsdDepth::Eight);

    assert_eq!(psd.color_mode(), ColorMode::Rgb);

    Ok(())
}
