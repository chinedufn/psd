use psd::Psd;
use failure::Error;

const BLEND_NORMAL_BLUE_RED_PIXEL: [u8; 4] = [85, 0, 170, 192];
const BLEND_MULTIPLY_BLUE_RED_PIXEL: [u8; 4] = [85, 0, 85, 192];
const BLEND_SCREEN_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 170, 192];
const BLEND_OVERLAY_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 85, 192];

const BLEND_DARKEN_BLUE_RED_PIXEL: [u8; 4] = [85, 0, 85, 192];
const BLEND_LIGHTEN_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 170, 192];

const BLEND_COLOR_BURN_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 85, 192];
const BLEND_COLOR_DODGE_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 85, 192];

const BLEND_LINEAR_BURN_BLUE_RED_PIXEL: [u8; 4] = [85, 0, 85, 192];
const BLEND_LINEAR_DODGE_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 170, 192];

const BLEND_HARD_LIGHT_BLUE_RED_PIXEL: [u8; 4] = [85, 0, 170, 192];
const BLEND_SOFT_LIGHT_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 85, 192];

const BLEND_DIFFERENCE_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 170, 192];
const BLEND_EXCLUSION_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 170, 192];

#[test]
fn normal() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-normal.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_NORMAL_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn multiply() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-multiply.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_MULTIPLY_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn screen() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-screen.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_SCREEN_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn overlay() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-overlay.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_OVERLAY_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn darken() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-darken.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_DARKEN_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn lighten() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-lighten.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_LIGHTEN_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn color_burn() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-color-burn.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_COLOR_BURN_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn color_dodge() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-color-dodge.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_COLOR_DODGE_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn linear_burn() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-linear-burn.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_LINEAR_BURN_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn linear_dodge() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-linear-dodge.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_LINEAR_DODGE_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn hard_light() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-hard-light.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_HARD_LIGHT_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn soft_light() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-soft-light.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_SOFT_LIGHT_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn difference() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-difference.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_DIFFERENCE_BLUE_RED_PIXEL);

    Ok(())
}

#[test]
fn exclusion() -> Result<(), Error> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-exclusion.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_EXCLUSION_BLUE_RED_PIXEL);

    Ok(())
}