use anyhow::Result;
use psd::ColorMode;
use psd::Psd;
use psd::PsdDepth;

/// cargo test --test channels one_channel_grayscale_raw_data -- --exact
#[test]
fn one_channel_grayscale_raw_data() -> Result<()> {
    let psd = include_bytes!("./fixtures/one-channel-1x1.psd");
    let psd = Psd::from_bytes(psd)?;

    assert_eq!(psd.color_mode(), ColorMode::Grayscale);
    assert_eq!(psd.depth(), PsdDepth::Sixteen);

    let final_image = psd.rgba();
    assert_eq!(final_image, [175, 175, 175, 255]);

    // There is one layer which should have the same RGBA as the final image
    let layer_rgba = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(layer_rgba, [175, 175, 175, 255]);

    Ok(())
}

/// Right now we just make sure that nothing throws when we try to parse a psd that
/// is 16 bit grayscale.
///
/// After exporting this PSD into a png from Photoshop, the colors appeared to be slightly off.
/// Similarly, our colors appear to be slightly off. Usually by 10 or so units (out of 255).
///
/// We can investigate this in further in the future.
///
/// cargo test --test channels two_channel_grayscale_raw_data -- --exact
#[test]
fn two_channel_grayscale_raw_data() -> Result<()> {
    let psd = include_bytes!("./fixtures/two-channel-8x8.psd");
    let psd = Psd::from_bytes(psd)?;

    assert_eq!(psd.color_mode(), ColorMode::Grayscale);
    assert_eq!(psd.depth(), PsdDepth::Sixteen);

    // The 64th pixel in the image. So the top right corner pixel
    //
    // We used the eyedropper tool in photoshop to get the exact color of the top right pixel.
    // Verify that it appear here.
    // TODO:
    /*
    let top_right_pixel = 7 * 4;
    let expected_top_right = &[27, 27, 27, 255];

    let composite = psd.rgba();
    let composite_top_right = &composite[top_right_pixel..top_right_pixel + 4];

    let layer = psd.flatten_layers_rgba(&|_| true)?;
    let layer_top_right = &layer[top_right_pixel..top_right_pixel + 4];

    assert_eq!(composite_top_right, expected_top_right);
    assert_eq!(layer_top_right, expected_top_right);
    */

    Ok(())
}
