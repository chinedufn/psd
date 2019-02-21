use failure::Error;
use psd::{Psd, PsdLayerChannelCompression};

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];

#[test]
fn rle_decompress_final_image() -> Result<(), Error> {
    let psd = include_bytes!("./rle-3-layer-8x8.psd");
    let psd = Psd::from_bytes(psd)?;

    assert_eq!(
        psd.compression(),
        &PsdLayerChannelCompression::RleCompressed
    );

    let image_data_rgb = psd.rgb();
    let image_data_rgba = psd.rgba();

    // Final image is red since the top layer is red
    assert_eq!(image_data_rgb, make_8x8_rgb([0, 0, 255]));

    assert_eq!(image_data_rgba, make_blue_8x8_rgba());

    Ok(())
}

#[test]
fn rle_decompress_layer() -> Result<(), Error> {
    unimplemented!(
        r#"Verify that red layer has channels with Rle then get rgba for red layer\
    and make sure it is red 8x8"#
    );
}

fn make_red_8x8_rgba() -> Vec<u8> {
    make_8x8_rgba(RED_PIXEL)
}

fn make_blue_8x8_rgba() -> Vec<u8> {
    make_8x8_rgba(BLUE_PIXEL)
}

fn make_green_8x8() -> Vec<u8> {
    make_8x8_rgba(GREEN_PIXEL)
}

fn make_blue_8x8() -> Vec<u8> {
    make_8x8_rgba(BLUE_PIXEL)
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

fn make_8x8_rgb(color: [u8; 3]) -> Vec<u8> {
    let mut pixels = vec![];

    for _ in 0..8 * 8 {
        pixels.push(color[0]);
        pixels.push(color[1]);
        pixels.push(color[2]);
    }

    pixels
}
