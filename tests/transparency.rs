use anyhow::{anyhow, Result};
use psd::Psd;
use psd::PsdChannelCompression;
use psd::PsdChannelKind;
use std::collections::HashMap;

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
// const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];

// Transparent pixels in the image data section start [255, 255, 255, 0]
// const TRANSPARENT_PIXEL_IMAGE_DATA: [u8; 4] = [255, 255, 255, 0];

// In the layer and mask info section we fill in transparent rgba pixels ourselves as [0, 0, 0, 0]
// const TRANSPARENT_PIXEL_LAYER: [u8; 4] = [0, 0, 0, 0];

// Test that images that have transparent pixels and don't use compression
// return the correct RGBA
#[test]
fn transparency_raw_data() -> Result<()> {
    let psd = include_bytes!("./fixtures/3x3-opaque-center.psd");
    let psd = Psd::from_bytes(psd)?;

    let blue_pixels = vec![(1, 1, BLUE_PIXEL), (2, 0, BLUE_PIXEL)];

    assert_colors(psd.rgba(), &psd, &blue_pixels);

    assert_colors(
        psd.layer_by_name("OpaqueCenter")
            .ok_or(anyhow!("layer not found"))?
            .rgba(),
        &psd,
        &blue_pixels,
    );

    Ok(())
}

// Test that images that have transparent pixels and use rle compression
// return the correct RGBA
#[test]
fn transparency_rle_compressed() -> Result<()> {
    let psd = include_bytes!("./fixtures/16x16-rle-partially-opaque.psd");
    let psd = Psd::from_bytes(psd)?;

    let mut red_block = vec![];
    for left in 0..9 {
        for top in 0..9 {
            red_block.push((left + 1, top + 1, RED_PIXEL));
        }
    }

    assert_eq!(psd.compression(), &PsdChannelCompression::RleCompressed);

    assert_colors(psd.rgba(), &psd, &red_block);

    assert_eq!(
        psd.layer_by_name("OpaqueCenter")
            .ok_or(anyhow!("layer not found"))?
            .compression(PsdChannelKind::Red)?,
        PsdChannelCompression::RleCompressed
    );

    assert_colors(
        psd.layer_by_name("OpaqueCenter")
            .ok_or(anyhow!("layer not found"))?
            .rgba(),
        &psd,
        &red_block,
    );

    Ok(())
}

// Fixes an `already borrowed: BorrowMutError` that we were getting in the `flattened_pixel`
// method when we were recursing into the method and trying to borrow when we'd already borrowed.
#[test]
fn transparent_above_opaque() -> Result<()> {
    let psd = include_bytes!("./fixtures/transparent-above-opaque.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;

    assert_eq!(image[0..4], BLUE_PIXEL);

    Ok(())
}

// Ensure that the specified, zero-indexed left, top coordinate has the provided pixel color.
// Otherwise it should be fully transparent.
// (left, top, pixel)
fn assert_colors(image: Vec<u8>, psd: &Psd, assertions: &[(usize, usize, [u8; 4])]) {
    let pixel_count = (psd.width() * psd.height()) as usize;
    let width = psd.width() as usize;

    let mut asserts = HashMap::new();
    for assertion in assertions {
        asserts.insert((assertion.0, assertion.1), assertion.2);
    }

    for idx in 0..pixel_count {
        let left = idx % width;
        let top = idx / width;

        let pixel_color = &image[idx * 4..idx * 4 + 4];

        match asserts.get(&(left, top)) {
            Some(expected_color) => {
                assert_eq!(expected_color, pixel_color);
            }
            None => {
                assert_eq!(pixel_color[3], 0, "Pixel should be transparent");
            }
        };
    }
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

fn put_pixel(image: &mut Vec<u8>, width: usize, left: usize, top: usize, new: [u8; 4]) {
    let idx = (top * width) + left;
    image[idx * 4] = new[0];
    image[idx * 4 + 1] = new[1];
    image[idx * 4 + 2] = new[2];
    image[idx * 4 + 3] = new[3];
}
