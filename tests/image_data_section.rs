use psd::Psd;

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];

#[test]
fn image_data_section() {
    let psd = include_bytes!("./fixtures/two-layers-red-green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(&psd.rgba(), &RED_PIXEL);
}
