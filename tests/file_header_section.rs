use failure::Error;
use psd::PsdDepth;
use psd::{ColorMode, Psd};

#[test]
fn file_header_section() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/green-1x1.psd");

    let psd = Psd::from_bytes(psd)?;

    assert_eq!(psd.width(), 1);
    assert_eq!(psd.height(), 1);

    assert_eq!(psd.depth(), PsdDepth::Eight);

    assert_eq!(psd.color_mode(), ColorMode::Rgb);

    Ok(())
}
