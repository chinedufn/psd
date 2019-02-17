use psd::{ColorMode, Psd};

const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];

#[test]
fn layer_and_mask_information_section() {
    let psd = include_bytes!("./green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);

    // TODO: BREADCRUMB -> type out a test based on how I think that this should work..
    // Then add enough implementation and unit tests to make it happen
    //    psd.layers() -> HashMap<String, PsdLayer>;
    let layer = psd.layers().get("First Layer").unwrap();

    assert_eq!(layer.pixels(), &GREEN_PIXEL);
}
