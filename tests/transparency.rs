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

    let psd = Psd::from_bytes(psd)?;
    let width = psd.width() as usize;
    let pixel_count = psd.width() * psd.height();

    let mut expected_image_data = make_image(TRANSPARENT_PIXEL_IMAGE_DATA, pixel_count);
    replace_pixel(&mut expected_image_data, width, 1, 1, BLUE_PIXEL);

    assert_eq!(psd.rgba(), expected_image_data);

    let mut expected_layer = make_image(TRANSPARENT_PIXEL_LAYER, pixel_count);
    replace_pixel(&mut expected_layer, width, 1, 1, BLUE_PIXEL);

    assert_eq!(psd.layer_by_name("OpaqueCenter")?.rgba()?, expected_layer);

    Ok(())
}

// Test that images that have transparent pixels and use rle compression
// return the correct RGBA
#[test]
fn transparency_rle_compressed() -> Result<(), failure::Error> {
    let psd = include_bytes!("./9x9-rle-opaque-center.psd");

    let psd = Psd::from_bytes(psd)?;
    let width = psd.width() as usize;
    let pixel_count = psd.width() * psd.height();

    let mut expected_image_data = make_image(TRANSPARENT_PIXEL_IMAGE_DATA, pixel_count);
    replace_pixel(&mut expected_image_data, width, 4, 4, RED_PIXEL);

    assert_eq!(psd.rgba(), expected_image_data);

    let mut expected_layer = make_image(TRANSPARENT_PIXEL_LAYER, pixel_count);
    replace_pixel(&mut expected_layer, width, 4, 4, RED_PIXEL);

    assert_eq!(psd.layer_by_name("OpaqueCenter")?.rgba()?, expected_layer);

    Ok(())
}

fn make_image(pixel: [u8; 4], pixel_count: u32) -> Vec<u8> {
    let pixel_count = pixel_count as usize;
    let mut image = vec![0; pixel_count * 4];

    for idx in 0..pixel_count {
        image[idx * 4] = pixel[0];
        image[idx * 4 + 1] = pixel[1];
        image[idx * 4 + 2] = pixel[2];
        image[idx * 4 + 3] = pixel[3];
    }

    image
}

fn replace_pixel(image: &mut Vec<u8>, width: usize, left: usize, top: usize, new: [u8; 4]) {
    let idx = (top * width) + left;
    image[idx * 4] = new[0];
    image[idx * 4 + 1] = new[1];
    image[idx * 4 + 2] = new[2];
    image[idx * 4 + 3] = new[3];
}
