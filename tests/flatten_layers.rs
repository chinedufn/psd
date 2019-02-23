use psd::Psd;

const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];
const _GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];
const BLUE_PIXEL: [u8; 4] = [0, 0, 255, 255];


// A fully transparent pixel gets replaced by the pixel on the layer below it
#[test]
fn flatten_fully_transparent_pixel_replaced_by_pixel_below() -> Result<(), failure::Error> {
    let psd = include_bytes!("./transparent-top-layer-2x1.psd");
    let psd = Psd::from_bytes(psd)?;

    let flattened = psd.flatten_layers_rgba(&|(idx, layer)| {
        layer.name() == "Blue Layer" || layer.name() == "Red Layer"
    })?;

    assert_eq!(&flattened[0..4], &RED_PIXEL);
    assert_eq!(&flattened[4..8], &BLUE_PIXEL);

    Ok(())
}

