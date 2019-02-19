use psd::{ColorMode, Psd};

const RED_PIXEL: [u8; 3] = [255, 0, 0];

#[test]
fn image_data_section() {
    let psd = include_bytes!("./two-layers-red-green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.rgb(), &RED_PIXEL);
}
