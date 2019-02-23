use psd::Psd;

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];

// Transparent pixels in the image data section start [255, 255, 255, 0]
const TRANSPARENT_PIXEL_IMAGE_DATA: [u8; 4] = [255, 255, 255, 0];

// In the layer and mask info section we fill in transparent rgba pixels ourselves as [0, 0, 0, 0]
const TRANSPARENT_PIXEL_LAYER: [u8; 4] = [0, 0, 0, 0];

// Test that images that have transparent pixels and don't use compression
// return the correct RGBA
#[test]
fn transparency_raw_data() -> Result<(), failure::Error> {
    let psd = include_bytes!("./3x3-opaque-center.psd");
    // TODO: 9x9 image.. pixel in center not transparent.. one layer

    let psd = Psd::from_bytes(psd)?;

    let expected_image_data_pixels: Vec<u8> = vec![
        TRANSPARENT_PIXEL_IMAGE_DATA,
        TRANSPARENT_PIXEL_IMAGE_DATA,
        TRANSPARENT_PIXEL_IMAGE_DATA,
        TRANSPARENT_PIXEL_IMAGE_DATA,
        BLUE_PIXEL,
        TRANSPARENT_PIXEL_IMAGE_DATA,
        TRANSPARENT_PIXEL_IMAGE_DATA,
        TRANSPARENT_PIXEL_IMAGE_DATA,
        TRANSPARENT_PIXEL_IMAGE_DATA,
    ]
    .iter()
    .flat_map(|pixel| pixel.to_vec().into_iter())
    .collect();

    assert_eq!(psd.rgba(), expected_image_data_pixels);

    let expected_layer_pixels: Vec<u8> = vec![
        TRANSPARENT_PIXEL_LAYER,
        TRANSPARENT_PIXEL_LAYER,
        TRANSPARENT_PIXEL_LAYER,
        TRANSPARENT_PIXEL_LAYER,
        BLUE_PIXEL,
        TRANSPARENT_PIXEL_LAYER,
        TRANSPARENT_PIXEL_LAYER,
        TRANSPARENT_PIXEL_LAYER,
        TRANSPARENT_PIXEL_LAYER,
    ]
    .iter()
    .flat_map(|pixel| pixel.to_vec().into_iter())
    .collect();

    assert_eq!(
        psd.layer_by_name("OpaqueCenter")?.rgba()?,
        expected_layer_pixels
    );

    Ok(())
}

// Test that images that have transparent pixels and use rle compression
// return the correct RGBA
#[test]
fn transparency_rle_compressed() -> Result<(), failure::Error> {
    unimplemented!()
}
