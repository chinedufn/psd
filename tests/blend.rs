//! FIXME: Combine these all into one test that iterates through a vector of
//! (PathBuf, [f32; 4])

use anyhow::Result;
use psd::Psd;

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
const BLEND_VIVID_LIGHT_BLUE_RED_PIXEL: [u8; 4] = [85, 0, 170, 192];
const BLEND_LINEAR_LIGHT_BLUE_RED_PIXEL: [u8; 4] = [85, 0, 169, 192];
const BLEND_PIN_LIGHT_BLUE_RED_PIXEL: [u8; 4] = [85, 0, 170, 192];
const BLEND_HARD_MIX_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 85, 192];

const BLEND_SUBTRACT_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 85, 192];
const BLEND_DIVIDE_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 85, 192];

const BLEND_DIFFERENCE_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 170, 192];
const BLEND_EXCLUSION_BLUE_RED_PIXEL: [u8; 4] = [170, 0, 170, 192];

// cargo test --test blend normal -- --exact
/// Verify that flattening layers produces the expected pixel for `Normal` blend mode.
#[test]
fn normal() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-normal.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_NORMAL_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend multiply -- --exact
/// Verify that flattening layers produces the expected pixel for `Multiply` blend mode.
#[test]
fn multiply() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-multiply.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_MULTIPLY_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend screen -- --exact
/// Verify that flattening layers produces the expected pixel for `Screen` blend mode.
#[test]
fn screen() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-screen.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_SCREEN_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend overlay -- --exact
/// Verify that flattening layers produces the expected pixel for `Overlay` blend mode.
#[test]
fn overlay() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-overlay.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_OVERLAY_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend darken -- --exact
/// Verify that flattening layers produces the expected pixel for `Darken` blend mode.
#[test]
fn darken() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-darken.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_DARKEN_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend lighten -- --exact
/// Verify that flattening layers produces the expected pixel for `Lighten` blend mode.
#[test]
fn lighten() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-lighten.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_LIGHTEN_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend color_burn -- --exact
/// Verify that flattening layers produces the expected pixel for `Color Burn` blend mode.
#[test]
fn color_burn() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-color-burn.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_COLOR_BURN_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend color_dodge -- --exact
/// Verify that flattening layers produces the expected pixel for `Color Dodge` blend mode.
#[test]
fn color_dodge() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-color-dodge.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_COLOR_DODGE_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend linear_burn -- --exact
/// Verify that flattening layers produces the expected pixel for `Linear Burn` blend mode.
#[test]
fn linear_burn() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-linear-burn.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_LINEAR_BURN_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend linear_dodge -- --exact
/// Verify that flattening layers produces the expected pixel for `Linear Dodge` blend mode.
#[test]
fn linear_dodge() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-linear-dodge.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_LINEAR_DODGE_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend hard_light -- --exact
/// Verify that flattening layers produces the expected pixel for `Hard Light` blend mode.
#[test]
fn hard_light() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-hard-light.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_HARD_LIGHT_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend soft_light -- --exact
/// Verify that flattening layers produces the expected pixel for `Soft Light` blend mode.
#[test]
fn soft_light() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-soft-light.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_SOFT_LIGHT_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend divide -- --exact
/// Verify that flattening layers produces the expected pixel for `Divide` blend mode.
#[test]
fn divide() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-divide.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_DIVIDE_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend subtract -- --exact
/// Verify that flattening layers produces the expected pixel for `Subtract` blend mode.
#[test]
fn subtract() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-subtract.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_SUBTRACT_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend difference -- --exact
/// Verify that flattening layers produces the expected pixel for `Difference` blend mode.
#[test]
fn difference() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-difference.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_DIFFERENCE_BLUE_RED_PIXEL);

    Ok(())
}

// cargo test --test blend exclusion -- --exact
/// Verify that flattening layers produces the expected pixel for `Exclusion` blend mode.
#[test]
fn exclusion() -> Result<()> {
    let psd = include_bytes!("./fixtures/blending/blue-red-1x1-exclusion.psd");
    let psd = Psd::from_bytes(psd)?;

    let image = psd.flatten_layers_rgba(&|_| true)?;
    assert_eq!(image[0..4], BLEND_EXCLUSION_BLUE_RED_PIXEL);

    Ok(())
}
