use psd::Psd;

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];

// cargo test --test image_data_section image_data_section -- --exact
/// Verify that the image data section is decoded correctly for a simple two-layer 1x1 fixture.
#[test]
fn image_data_section() {
    let psd = include_bytes!("./fixtures/two-layers-red-green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(&psd.rgba(), &RED_PIXEL);
}
