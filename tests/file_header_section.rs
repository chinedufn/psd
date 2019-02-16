use psd::{ColorMode, Psd};

#[test]
fn file_section_header() {
    let psd = include_bytes!("./green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.width(), 1);
    assert_eq!(psd.height(), 1);

    assert_eq!(psd.depth(), 8);

    assert_eq!(psd.color_mode(), ColorMode::Rgb);
}
