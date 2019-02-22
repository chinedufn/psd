use psd::Psd;

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const _GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];

// FIXME: Create tests for transparency.rs
// Create a 3x3 image with a single colored in blue pixel in the middle
// and the rest transparent. Verify that `rgba` for all layers returns properly.
// As well as `rgb` and `rgba` for the final image. THis makes sure that we properly
// account for layer pixel data starting from the first non-transparent pixel

// FIXME:
#[test]
fn flatten_transparent_pixel_replaced_by_pixel_below() -> Result<(), failure::Error> {
    let psd = include_bytes!("./transparent-top-layer-2x1.psd");
    let psd = Psd::from_bytes(psd)?;

    let flattened = psd.flatten_layers_rgba(&|(idx, layer)| {
        layer.name() == "Blue Layer" || layer.name() == "Red Layer"
    })?;

    assert_eq!(&flattened[0..4], &RED_PIXEL);
    assert_eq!(&flattened[4..8], &BLUE_PIXEL);

    panic!("");

    Ok(())
}