use psd::Psd;

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];
const TRANSPARENT_PIXEL: [u8; 4] = [255, 255, 255, 0];

// Test that images that have transparent pixels and don't use compression
// return the correct RGBA
#[test]
fn transparency_raw_data() -> Result<(), failure::Error> {
    let psd = include_bytes!("./3x3-opaque-center.psd");
    // TODO: 9x9 image.. pixel in center not transparent.. one layer

    let psd = Psd::from_bytes(psd)?;

    let expected_pixels: Vec<u8> = vec![
        TRANSPARENT_PIXEL,
        TRANSPARENT_PIXEL,
        TRANSPARENT_PIXEL,
        TRANSPARENT_PIXEL,
        BLUE_PIXEL,
        TRANSPARENT_PIXEL,
        TRANSPARENT_PIXEL,
        TRANSPARENT_PIXEL,
        TRANSPARENT_PIXEL,
    ]
    .iter()
    .flat_map(|pixel| pixel.to_vec().into_iter())
    .collect();

    assert_eq!(psd.rgba(), expected_pixels);

    assert_eq!(psd.layer_by_name("OpaqueCenter")?.rgba()?, expected_pixels);

    Ok(())
}

// Test that images that have transparent pixels and use rle compression
// return the correct RGBA
#[test]
fn transparency_rle_compressed() -> Result<(), failure::Error> {
    unimplemented!()
}