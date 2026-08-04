use anyhow::Result;
use psd::Psd;

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const _GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];

// cargo test --test flatten_layers flatten_fully_transparent_pixel_replaced_by_pixel_below -- --exact
/// Verify that flattening replaces a fully transparent pixel with the pixel from the layer below.
#[test]
fn flatten_fully_transparent_pixel_replaced_by_pixel_below() -> Result<()> {
    let psd = include_bytes!("./fixtures/transparent-top-layer-2x1.psd");
    let psd = Psd::from_bytes(psd)?;

    let flattened = psd.flatten_layers_rgba(&|(_, layer)| {
        layer.name() == "Blue Layer" || layer.name() == "Red Layer"
    })?;

    assert_eq!(&flattened[0..4], &RED_PIXEL);
    assert_eq!(&flattened[4..8], &BLUE_PIXEL);

    Ok(())
}

// cargo test --test flatten_layers no_matching_layers -- --exact
/// Verify that flattening with a filter that matches zero layers returns a fully transparent image.
#[test]
fn no_matching_layers() -> Result<()> {
    let psd = include_bytes!("./fixtures/transparent-top-layer-2x1.psd");
    let psd = Psd::from_bytes(psd)?;

    let flattened = psd.flatten_layers_rgba(&|(_, _)| false)?;

    assert_eq!(&flattened[0..8], &[0, 0, 0, 0, 0, 0, 0, 0]);

    Ok(())
}
