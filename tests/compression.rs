use anyhow::Result;
use psd::{Psd, PsdChannelCompression};

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];

/// cargo test --test compression rle_decompress_final_image -- --exact
#[test]
fn rle_decompress_final_image() -> Result<()> {
    let psd = include_bytes!("./fixtures/rle-3-layer-8x8.psd");
    let psd = Psd::from_bytes(psd)?;

    assert_eq!(psd.compression(), &PsdChannelCompression::RleCompressed);

    let image_data_rgba = psd.rgba();

    // Final image is blue since the top layer is blue
    assert_eq!(image_data_rgba, make_blue_8x8_rgba());

    Ok(())
}

/// cargo test --test compression rle_decompress_layer -- --exact
#[test]
fn rle_decompress_layer() -> Result<()> {
    let psd = include_bytes!("./fixtures/rle-3-layer-8x8.psd");
    let psd = Psd::from_bytes(psd)?;

    for (layer_name, expected_pixels) in [
        ("Red Layer", make_red_8x8_rgba()),
        ("Green Layer", make_green_8x8()),
        ("Blue Layer", make_blue_8x8_rgba()),
    ]
    .iter()
    {
        test_rle_layer(&psd, &layer_name, expected_pixels);
    }

    Ok(())
}

fn test_rle_layer(psd: &Psd, layer_name: &str, expected_pixels: &[u8]) {
    let layer = psd.layer_by_name(layer_name).unwrap();
    assert_eq!(&layer.rgba().as_slice(), &expected_pixels);
}

// Below are methods to make different expected final pixels so that we can text our generated
// pixels against these expected pixels below.

fn make_red_8x8_rgba() -> Vec<u8> {
    make_8x8_rgba(RED_PIXEL)
}

fn make_blue_8x8_rgba() -> Vec<u8> {
    make_8x8_rgba(BLUE_PIXEL)
}

fn make_green_8x8() -> Vec<u8> {
    make_8x8_rgba(GREEN_PIXEL)
}

fn make_8x8_rgba(color: [u8; 4]) -> Vec<u8> {
    let mut pixels = vec![];

    for _ in 0..8 * 8 {
        pixels.push(color[0]);
        pixels.push(color[1]);
        pixels.push(color[2]);
        pixels.push(color[3]);
    }

    pixels
}
