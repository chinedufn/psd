use failure::Error;
use psd::Psd;

#[test]
fn one_channel_raw_data() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/one-channel-1x1.psd");
    let psd = Psd::from_bytes(psd)?;

    let final_image = psd.rgba();
    assert_eq!(final_image, [175, 175, 175, 255]);

    // There is one layer which should have the same RGBA as the final image
    let layer_rgba = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(layer_rgba, [175, 175, 175, 255]);

    Ok(())
}
