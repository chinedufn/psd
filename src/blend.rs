use crate::sections::layer_and_mask_information_section::layer::BlendMode;

pub(crate) type Pixel = [u8; 4];

// Multiplies the pixel's current alpha by the passed in `opacity`
pub(crate) fn apply_opacity(pixel: &mut Pixel, opacity: u8) {
    let alpha = opacity as f32 / 255.;
    pixel[3] = (pixel[3] as f32 * alpha) as u8;
}

const INV_255: f32 = 1.0 / 255.0;
const EPSILON: f32 = 1e-6;

///
/// https://www.w3.org/TR/compositing-1/#simplealphacompositing
/// `Cs = (1 - αb) x Cs + αb x B(Cb, Cs)`
/// `cs = Cs x αs`
/// `cb = Cb x αb`
/// `co = cs + cb x (1 - αs)`
/// Where
///  - Cs: is the source color
///  - Cb: is the backdrop color
///  - αs: is the source alpha
///  - αb: is the backdrop alpha
///  - B(Cb, Cs): is the mixing function
///
/// `αo = αs + αb x (1 - αs)`
/// Where
///  - αo: the alpha value of the composite
///  - αs: the alpha value of the graphic element being composited
///  - αb: the alpha value of the backdrop
///
/// Final:
/// `Co = co / αo`
///
/// *The backdrop is the content behind the element and is what the element is composited with. This means that the backdrop is the result of compositing all previous elements.
///
/// ** this optimized version is based on tdakkota's implementation but improves it by ~10%-25% by removing division where possible and allows for the blending of methods that need access to all channels simultaneously.
pub(crate) fn blend_pixels(
    top: Pixel,
    bottom: Pixel,
    blend_mode: BlendMode,
    out: &mut Pixel,
) {
    let alpha_s = top[3] as f32 * INV_255;
    let alpha_b = bottom[3] as f32 * INV_255;
    let alpha_output = alpha_s + alpha_b - alpha_s * alpha_b;

    if alpha_output > 0.0 {
        let (r_s, g_s, b_s) = (
            top[0] as f32 * INV_255,
            top[1] as f32 * INV_255,
            top[2] as f32 * INV_255,
        );
        let (r_b, g_b, b_b) = (
            bottom[0] as f32 * INV_255,
            bottom[1] as f32 * INV_255,
            bottom[2] as f32 * INV_255,
        );

        let (r_co, g_co, b_co) = match blend_mode {
            BlendMode::DarkerColor => {
                let lum_s = luminance(r_s, g_s, b_s);
                let lum_b = luminance(r_b, g_b, b_b);
                let alpha_s_255 = alpha_s * 255.0;
                let alpha_b_255 = alpha_b * 255.0;

                if lum_s < lum_b {
                    (r_s * alpha_s_255, g_s * alpha_s_255, b_s * alpha_s_255)
                } else {
                    (r_b * alpha_b_255, g_b * alpha_b_255, b_b * alpha_b_255)
                }
            }
            BlendMode::LighterColor => {
                let lum_s = luminance(r_s, g_s, b_s);
                let lum_b = luminance(r_b, g_b, b_b);
                let alpha_s_255 = alpha_s * 255.0;
                let alpha_b_255 = alpha_b * 255.0;

                if lum_s > lum_b {
                    (r_s * alpha_s_255, g_s * alpha_s_255, b_s * alpha_s_255)
                } else {
                    (r_b * alpha_b_255, g_b * alpha_b_255, b_b * alpha_b_255)
                }
            }
            BlendMode::Hue => {
                let (r, g, b) = blend_hue(r_b, g_b, b_b, r_s, g_s, b_s);
                (r * 255.0, g * 255.0, b * 255.0)
            }
            BlendMode::Saturation => {
                let (r, g, b) = blend_saturation(r_b, g_b, b_b, r_s, g_s, b_s);
                (r * 255.0, g * 255.0, b * 255.0)
            }
            BlendMode::Color => {
                let (r, g, b) = blend_color(r_b, g_b, b_b, r_s, g_s, b_s);
                (r * 255.0, g * 255.0, b * 255.0)
            }
            BlendMode::Luminosity => {
                let (r, g, b) = blend_luminosity(r_b, g_b, b_b, r_s, g_s, b_s);
                (r * 255.0, g * 255.0, b * 255.0)
            }
            _ => {
                let blend_f = map_blend_mode(blend_mode);

                (
                    composite(r_s, alpha_s, r_b, alpha_b, blend_f) * 255.0,
                    composite(g_s, alpha_s, g_b, alpha_b, blend_f) * 255.0,
                    composite(b_s, alpha_s, b_b, alpha_b, blend_f) * 255.0,
                )
            }
        };

        // Divide after rounding, matching the original function's order
        out[0] = (r_co.round() / alpha_output).clamp(0.0, 255.0) as u8;
        out[1] = (g_co.round() / alpha_output).clamp(0.0, 255.0) as u8;
        out[2] = (b_co.round() / alpha_output).clamp(0.0, 255.0) as u8;
        out[3] = (alpha_output * 255.0).round().clamp(0.0, 255.0) as u8;
    } else {
        out.fill(0);
    }
}


type BlendFunction = dyn Fn(f32, f32) -> f32;

/// Returns blend function for given BlendMode
fn map_blend_mode(blend_mode: BlendMode) -> &'static BlendFunction {
    // Modes are sorted like in Photoshop UI
    // TODO: Finish dissolve
    match blend_mode {
        BlendMode::PassThrough => &pass_through, // only for groups
        // --------------------------------------
        BlendMode::Normal => &normal,
        BlendMode::Dissolve => &dissolve,
        // --------------------------------------
        BlendMode::Darken => &darken,
        BlendMode::Multiply => &multiply,
        BlendMode::ColorBurn => &color_burn,
        BlendMode::LinearBurn => &linear_burn,
        BlendMode::DarkerColor => &dummy_blend, // handled in blend_pixels
        // --------------------------------------
        BlendMode::Lighten => &lighten,
        BlendMode::Screen => &screen,
        BlendMode::ColorDodge => &color_dodge,
        BlendMode::LinearDodge => &linear_dodge,
        BlendMode::LighterColor => &dummy_blend, // handled in blend_pixels
        // --------------------------------------
        BlendMode::Overlay => &overlay,
        BlendMode::SoftLight => &soft_light,
        BlendMode::HardLight => &hard_light,
        BlendMode::VividLight => &vivid_light,
        BlendMode::LinearLight => &linear_light,
        BlendMode::PinLight => &pin_light,
        BlendMode::HardMix => &hard_mix,
        // --------------------------------------
        BlendMode::Difference => &difference,
        BlendMode::Exclusion => &exclusion,
        BlendMode::Subtract => &subtract,
        BlendMode::Divide => &divide,
        // --------------------------------------
        BlendMode::Hue => &dummy_blend, // handled in blend_pixels
        BlendMode::Saturation => &dummy_blend, // handled in blend_pixels
        BlendMode::Color => &dummy_blend, // handled in blend_pixels
        BlendMode::Luminosity => &dummy_blend, // handled in blend_pixels
    }
}

/// fake function for modes handled directly in `blend_pixels`.
fn dummy_blend(_color_b: f32, _color_s: f32) -> f32 {
    0.0
}

fn pass_through(_color_b: f32, color_s: f32) -> f32 {
    color_s
}

/// https://www.w3.org/TR/compositing-1/#blendingnormal
/// This is the default attribute which specifies no blending. The blending formula simply selects the source color.
///
/// `B(Cb, Cs) = Cs`
#[inline(always)]
fn normal(_color_b: f32, color_s: f32) -> f32 {
    color_s
}

fn dissolve(_color_b: f32, _color_s: f32) -> f32 {
    unimplemented!()
}

// Darken modes

/// https://www.w3.org/TR/compositing-1/#blendingdarken
/// Selects the darker of the backdrop and source colors.
///
/// The backdrop is replaced with the source where the source is darker; otherwise, it is left unchanged.
///
/// `B(Cb, Cs) = min(Cb, Cs)`
#[inline(always)]
fn darken(color_b: f32, color_s: f32) -> f32 {
    color_b.min(color_s)
}

/// https://www.w3.org/TR/compositing-1/#blendingmultiply
/// The source color is multiplied by the destination color and replaces the destination.
/// The resultant color is always at least as dark as either the source or destination color.
/// Multiplying any color with black results in black. Multiplying any color with white preserves the original color.
///
/// `B(Cb, Cs) = Cb x Cs`
#[inline(always)]
fn multiply(color_b: f32, color_s: f32) -> f32 {
    color_b * color_s
}

/// https://www.w3.org/TR/compositing-1/#blendingcolorburn
///
/// Darkens the backdrop color to reflect the source color. Painting with white produces no change.
///
/// ```text
/// if(Cb == 1)
///     B(Cb, Cs) = 1
/// else
///     B(Cb, Cs) = max(0, (1 - (1 - Cs) / Cb))
///```
#[inline(always)]
fn color_burn(color_b: f32, color_s: f32) -> f32 {
    // Why Use Epsilon Comparisons:
    // Floating-Point Precision: Accounts for tiny deviations due to floating-point arithmetic.
    // Robustness: Prevents unexpected behavior when color_b or color_s are very close to the comparison values.
    // Accuracy: Ensures that values very close to 0.0 or 1.0 are treated appropriately.
    if (color_b - 1.0).abs() < EPSILON {
        1.0
    } else if color_b.abs() < EPSILON {
        // the previous method could result in a division by zero
        0.0
    } else {
        (1.0 - (1.0 - color_s) / color_b).max(0.0)
    }
}

/// See: http://www.simplefilter.de/en/basics/mixmods.html
/// psd_tools impl: https://github.com/psd-tools/psd-tools/blob/master/src/psd_tools/composer/blend.py#L139
///
/// This variant of subtraction is also known as subtractive color blending.
/// The tonal values of fore- and background that sum up to less than 255 (i.e. 1.0) become pure black.
/// If the foreground image A is converted prior to the operation, the result is the mathematical subtraction.
///
/// `B(Cb, Cs) = max(0,  Cb + Cs - 1)`
#[inline(always)]
fn linear_burn(color_b: f32, color_s: f32) -> f32 {
    (color_b + color_s - 1.).max(0.)
}

// Lighten modes

/// https://www.w3.org/TR/compositing-1/#blendinglighten
/// Selects the lighter of the backdrop and source colors.
///
/// The backdrop is replaced with the source where the source is lighter; otherwise, it is left unchanged.
///
/// `B(Cb, Cs) = max(Cb, Cs)`
#[inline(always)]
fn lighten(color_b: f32, color_s: f32) -> f32 {
    color_b.max(color_s)
}

/// https://www.w3.org/TR/compositing-1/#blendingscreen
/// Multiplies the complements of the backdrop and source color values, then complements the result.
///
/// The result color is always at least as light as either of the two constituent colors.
/// Screening any color with white produces white; screening with black leaves the original color unchanged.
/// The effect is similar to projecting multiple photographic slides simultaneously onto a single screen.
///
/// `B(Cb, Cs) = 1 - [(1 - Cb) x (1 - Cs)] = Cb + Cs - (Cb x Cs)`
#[inline(always)]
fn screen(color_b: f32, color_s: f32) -> f32 {
    color_b + color_s - (color_b * color_s)
}

/// https://www.w3.org/TR/compositing-1/#blendingcolordodge
///
/// Brightens the backdrop color to reflect the source color. Painting with black produces no changes.
///
/// ```text
/// if(Cb == 0)
///     B(Cb, Cs) = 0
/// else if(Cs == 1)
///     B(Cb, Cs) = 1
/// else
///     B(Cb, Cs) = min(1, Cb / (1 - Cs))
/// ```
#[inline(always)]
fn color_dodge(color_b: f32, color_s: f32) -> f32 {
    if color_b.abs() < EPSILON {
        0.0
    } else if (color_s - 1.0).abs() < EPSILON {
        1.0
    } else {
        (color_b / (1.0 - color_s)).min(1.0)
    }
}

/// See: http://www.simplefilter.de/en/basics/mixmods.html
///
/// Adds the tonal values of fore- and background.
///
/// Also: Add
/// `B(Cb, Cs) = Cb + Cs`
#[inline(always)]
fn linear_dodge(color_b: f32, color_s: f32) -> f32 {
    (color_b + color_s).min(1.)
}

// Contrast modes

/// https://www.w3.org/TR/compositing-1/#blendingoverlay
/// Multiplies or screens the colors, depending on the backdrop color value.
///
/// Source colors overlay the backdrop while preserving its highlights and shadows.
/// The backdrop color is not replaced but is mixed with the source color to reflect the lightness or darkness of the backdrop.
///
/// `B(Cb, Cs) = HardLight(Cs, Cb)`
/// Overlay is the inverse of the hard-light blend mode. See the definition of hard-light for the formula.
#[inline(always)]
fn overlay(color_b: f32, color_s: f32) -> f32 {
    hard_light(color_s, color_b) // inverted hard_light
}

/// https://www.w3.org/TR/compositing-1/#blendingsoftlight
///
/// Darkens or lightens the colors, depending on the source color value.
/// The effect is similar to shining a diffused spotlight on the backdrop.
///
/// ```text
/// if(Cs <= 0.5)
///     B(Cb, Cs) = Cb - (1 - 2 x Cs) x Cb x (1 - Cb)
/// else
///     B(Cb, Cs) = Cb + (2 x Cs - 1) x (D(Cb) - Cb)
/// ```
/// with
/// ```text
/// if(Cb <= 0.25)
///     D(Cb) = ((16 * Cb - 12) x Cb + 4) x Cb
/// else
///     D(Cb) = sqrt(Cb)
/// ```
fn soft_light(color_b: f32, color_s: f32) -> f32 {
    // FIXME: this function uses W3C algorithm which is differ from Photoshop's algorithm
    // See: https://en.wikipedia.org/wiki/Blend_modes#Soft_Light
    let d = if color_b <= 0.25 {
        ((16. * color_b - 12.) * color_b + 4.) * color_b
    } else {
        color_b.sqrt()
    };

    if color_s <= 0.5 {
        color_b - (1. - 2. * color_s) * color_b * (1. - color_b)
    } else {
        color_b + (2. * color_s - 1.) * (d - color_b)
    }
}

/// https://www.w3.org/TR/compositing-1/#blendinghardlight
///
/// Multiplies or screens the colors, depending on the source color value.
/// The effect is similar to shining a harsh spotlight on the backdrop.
///
/// ```text
/// if(Cs <= 0.5)
///     B(Cb, Cs) = Multiply(Cb, 2 x Cs) = 2 x Cb x Cs
/// else
///     B(Cb, Cs) = Screen(Cb, 2 x Cs -1)
/// ```
/// See the definition of `multiply` and `screen` for their formulas.
#[inline(always)]
fn hard_light(color_b: f32, color_s: f32) -> f32 {
    if color_s < 0.5 {
        multiply(color_b, 2. * color_s)
    } else {
        screen(color_b, 2. * color_s - 1.)
    }
}

/// Applies the Vivid Light blending mode between two color components.
///
/// Vivid Light is a combination of `Color Burn` and `Color Dodge`:
/// * If the source color (`Cs`) is less than or equal to 0.5, `Color Burn` is applied.
/// * If the source color (`Cs`) is greater than 0.5, `Color Dodge` is applied.
///
/// # Formula:
/// ```text
/// if Cs <= 0.5:
///     B(Cb, Cs) = ColorBurn(Cb, 2 * Cs)
/// else:
///     B(Cb, Cs) = ColorDodge(Cb, 2 * (Cs - 0.5))
/// ```
///
/// # Arguments:
/// * `color_b` - The backdrop color component (0.0 to 1.0).
/// * `color_s` - The source color component (0.0 to 1.0).
///
/// # Returns:
/// The blended color component (0.0 to 1.0).
#[inline(always)]
fn vivid_light(color_b: f32, color_s: f32) -> f32 {
    if color_s <= 0.5 {
        color_burn(color_b, 2.0 * color_s)
    } else {
        color_dodge(color_b, 2.0 * (color_s - 0.5))
    }
}

/// Applies the Linear Light blending mode between two color components.
///
/// Linear Light is a combination of `Linear Burn` and `Linear Dodge`:
/// * If the source color (`Cs`) is less than or equal to 0.5, `Linear Burn` is applied.
/// * If the source color (`Cs`) is greater than 0.5, `Linear Dodge` is applied.
///
/// # Formula:
/// ```text
/// if Cs <= 0.5:
///     B(Cb, Cs) = LinearBurn(Cb, 2 * Cs)
/// else:
///     B(Cb, Cs) = LinearDodge(Cb, 2 * (Cs - 0.5))
/// ```
///
/// # Arguments:
/// * `color_b` - The backdrop color component (0.0 to 1.0).
/// * `color_s` - The source color component (0.0 to 1.0).
///
/// # Returns:
/// The blended color component (0.0 to 1.0).
#[inline(always)]
fn linear_light(color_b: f32, color_s: f32) -> f32 {
    if color_s <= 0.5 {
        linear_burn(color_b, 2.0 * color_s)
    } else {
        linear_dodge(color_b, 2.0 * (color_s - 0.5))
    }
}

/// Applies the Pin Light blending mode between two color components.
///
/// Pin Light is a combination of `Darken` and `Lighten`:
/// * If the source color (`Cs`) is less than or equal to 0.5, `Darken` is applied.
/// * If the source color (`Cs`) is greater than 0.5, `Lighten` is applied.
///
/// # Formula:
/// ```text
/// if Cs <= 0.5:
///     B(Cb, Cs) = Darken(Cb, 2 * Cs)
/// else:
///     B(Cb, Cs) = Lighten(Cb, 2 * (Cs - 0.5))
/// ```
///
/// # Arguments:
/// * `color_b` - The backdrop color component (0.0 to 1.0).
/// * `color_s` - The source color component (0.0 to 1.0).
///
/// # Returns:
/// The blended color component (0.0 to 1.0).
#[inline(always)]
fn pin_light(color_b: f32, color_s: f32) -> f32 {
    if color_s <= 0.5 {
        darken(color_b, 2.0 * color_s)
    } else {
        lighten(color_b, 2.0 * (color_s - 0.5))
    }
}

/// Applies the Hard Mix blending mode between two color components.
///
/// Hard Mix is a threshold-based mode that uses `Vivid Light` to calculate the blend result.
/// The result is binary, either fully black or fully white:
/// * If the result of `Vivid Light` is greater than or equal to 0.5, the result is 1.0 (white).
/// * Otherwise, the result is 0.0 (black).
///
/// # Formula:
/// ```text
/// result = VividLight(Cb, Cs)
/// if result >= 0.5:
///     B(Cb, Cs) = 1.0
/// else:
///     B(Cb, Cs) = 0.0
/// ```
///
/// # Arguments:
/// * `color_b` - The backdrop color component (0.0 to 1.0).
/// * `color_s` - The source color component (0.0 to 1.0).
///
/// # Returns:
/// The blended color component (0.0 or 1.0).
#[inline(always)]
fn hard_mix(color_b: f32, color_s: f32) -> f32 {
    let result = vivid_light(color_b, color_s);
    if result >= 0.5 {
        1.0
    } else {
        0.0
    }
}

// Inversion modes

/// https://www.w3.org/TR/compositing-1/#blendingdifference
///
/// Subtracts the darker of the two constituent colors from the lighter color.
/// Painting with white inverts the backdrop color; painting with black produces no change.
///
/// `B(Cb, Cs) = | Cb - Cs |`
#[inline(always)]
fn difference(color_b: f32, color_s: f32) -> f32 {
    (color_b - color_s).abs()
}

/// https://www.w3.org/TR/compositing-1/#blendingexclusion
///
/// Produces an effect similar to that of the Difference mode but lower in contrast.
/// Painting with white inverts the backdrop color; painting with black produces no change
///
/// `B(Cb, Cs) = Cb + Cs - 2 x Cb x Cs`
#[inline(always)]
fn exclusion(color_b: f32, color_s: f32) -> f32 {
    color_b + color_s - 2. * color_b * color_s
}

/// https://helpx.adobe.com/photoshop/using/blending-modes.html
///
/// Looks at the color information in each channel and subtracts the blend color from the base color.
///
/// `B(Cb, Cs) = Cb - Cs`
#[inline(always)]
fn subtract(color_b: f32, color_s: f32) -> f32 {
    (color_b - color_s).max(0.)
}

/// https://helpx.adobe.com/photoshop/using/blending-modes.html
///
/// Looks at the color information in each channel and divides the blend color from the base color.
/// In 8- and 16-bit images, any resulting negative values are clipped to zero.
///
/// `B(Cb, Cs) = Cb / Cs`
#[inline(always)]
fn divide(color_b: f32, color_s: f32) -> f32 {
    if color_s > 0. {
        color_b / color_s
    } else {
        color_b
    }
}

// Color component modes

/// Calculates the luminance of an RGB color using the Rec. 709 formula.
///
/// The luminance is a weighted sum of the red, green, and blue components to represent the brightness of a color.
/// The formula assigns higher importance to the green component, as human vision is more sensitive to green light.
///
/// # Formula:
/// `L = 0.2126 * R + 0.7152 * G + 0.0722 * B`
///
/// # Arguments:
/// * `r` - The red component of the color (0.0 to 1.0).
/// * `g` - The green component of the color (0.0 to 1.0).
/// * `b` - The blue component of the color (0.0 to 1.0).
///
/// # Returns:
/// The luminance of the RGB color as a floating-point value between 0.0 and 1.0.
fn luminance(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Converts an RGB color to its HSL (Hue, Saturation, Lightness) representation.
///
/// HSL separates color into three components:
/// * `Hue`: The type of color (0.0 to 1.0, where 0.0 is red, 0.33 is green, 0.67 is blue).
/// * `Saturation`: The intensity or purity of the color (0.0 is grayscale, 1.0 is fully saturated).
/// * `Lightness`: The brightness of the color (0.0 is black, 1.0 is white).
///
/// # Arguments:
/// * `r` - The red component of the color (0.0 to 1.0).
/// * `g` - The green component of the color (0.0 to 1.0).
/// * `b` - The blue component of the color (0.0 to 1.0).
///
/// # Returns:
/// A tuple containing:
/// * `h`: The hue (0.0 to 1.0).
/// * `s`: The saturation (0.0 to 1.0).
/// * `l`: The lightness (0.0 to 1.0).
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let l = (max + min) / 2.0;

    let s = if delta.abs() < EPSILON {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    let h = if delta.abs() < EPSILON {
        0.0
    } else if (max - r).abs() < EPSILON {
        ((g - b) / delta) % 6.0
    } else if (max - g).abs() < EPSILON {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    } / 6.0;

    let h = if h < 0.0 { h + 1.0 } else { h };

    (h, s, l)
}

/// Converts an HSL (Hue, Saturation, Lightness) color back to its RGB representation.
///
/// This function converts an HSL color (often used for color adjustments) into its RGB equivalent, where each component
/// is a floating-point value between 0.0 and 1.0.
///
/// # Arguments:
/// * `h` - The hue of the color (0.0 to 1.0, where 0.0 is red, 0.33 is green, 0.67 is blue).
/// * `s` - The saturation of the color (0.0 is grayscale, 1.0 is fully saturated).
/// * `l` - The lightness of the color (0.0 is black, 1.0 is white).
///
/// # Returns:
/// A tuple containing:
/// * `r`: The red component of the color (0.0 to 1.0).
/// * `g`: The green component of the color (0.0 to 1.0).
/// * `b`: The blue component of the color (0.0 to 1.0).
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < EPSILON {
        return (l, l, l);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let hk = h;
    let t = |offset: f32| -> f32 {
        let mut tc = hk + offset;
        if tc < 0.0 {
            tc += 1.0;
        }
        if tc > 1.0 {
            tc -= 1.0;
        }
        if tc < 1.0 / 6.0 {
            p + (q - p) * 6.0 * tc
        } else if tc < 0.5 {
            q
        } else if tc < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - tc) * 6.0
        } else {
            p
        }
    };

    let r = t(1.0 / 3.0);
    let g = t(0.0);
    let b = t(-1.0 / 3.0);

    (r, g, b)
}

/// Blends two RGB colors by preserving the hue of the source color and applying the saturation and lightness of the backdrop.
///
/// This blend mode is typically used to change the color tint of the backdrop without affecting its overall intensity or brightness.
///
/// # Arguments:
/// * `r_b`, `g_b`, `b_b` - The red, green, and blue components of the backdrop color.
/// * `r_s`, `g_s`, `b_s` - The red, green, and blue components of the source color.
///
/// # Returns:
/// A tuple containing the blended RGB values (r, g, b), where the hue of the source is applied to the backdrop.
fn blend_hue(r_b: f32, g_b: f32, b_b: f32, r_s: f32, g_s: f32, b_s: f32) -> (f32, f32, f32) {
    let (h_s, _, _) = rgb_to_hsl(r_s, g_s, b_s);
    let (_, s_b, l_b) = rgb_to_hsl(r_b, g_b, b_b);

    hsl_to_rgb(h_s, s_b, l_b)
}

/// Blends two RGB colors by preserving the saturation of the source color and applying the hue and lightness of the backdrop.
///
/// This blend mode is used to adjust the intensity of the color of the backdrop without affecting its hue or brightness.
///
/// # Arguments:
/// * `r_b`, `g_b`, `b_b` - The red, green, and blue components of the backdrop color.
/// * `r_s`, `g_s`, `b_s` - The red, green, and blue components of the source color.
///
/// # Returns:
/// A tuple containing the blended RGB values (r, g, b), where the saturation of the source is applied to the backdrop.
fn blend_saturation(r_b: f32, g_b: f32, b_b: f32, r_s: f32, g_s: f32, b_s: f32) -> (f32, f32, f32) {
    let (_, s_s, _) = rgb_to_hsl(r_s, g_s, b_s);
    let (h_b, _, l_b) = rgb_to_hsl(r_b, g_b, b_b);

    hsl_to_rgb(h_b, s_s, l_b)
}

/// Blends two RGB colors by preserving the hue and saturation of the source color and applying the lightness of the backdrop.
///
/// This blend mode is used to apply the color of the source to the backdrop while maintaining the overall brightness of the backdrop.
///
/// # Arguments:
/// * `r_b`, `g_b`, `b_b` - The red, green, and blue components of the backdrop color.
/// * `r_s`, `g_s`, `b_s` - The red, green, and blue components of the source color.
///
/// # Returns:
/// A tuple containing the blended RGB values (r, g, b), where the hue and saturation of the source are applied to the backdrop.
fn blend_color(r_b: f32, g_b: f32, b_b: f32, r_s: f32, g_s: f32, b_s: f32) -> (f32, f32, f32) {
    let (h_s, s_s, _) = rgb_to_hsl(r_s, g_s, b_s);
    let (_, _, l_b) = rgb_to_hsl(r_b, g_b, b_b);

    hsl_to_rgb(h_s, s_s, l_b)
}

/// Blends two RGB colors by preserving the luminosity (lightness) of the source color and applying the hue and saturation of the backdrop.
///
/// This blend mode is used to adjust the brightness of the backdrop to match that of the source without affecting its color hue or saturation.
///
/// # Arguments:
/// * `r_b`, `g_b`, `b_b` - The red, green, and blue components of the backdrop color.
/// * `r_s`, `g_s`, `b_s` - The red, green, and blue components of the source color.
///
/// # Returns:
/// A tuple containing the blended RGB values (r, g, b), where the luminosity of the source is applied to the backdrop.
fn blend_luminosity(r_b: f32, g_b: f32, b_b: f32, r_s: f32, g_s: f32, b_s: f32) -> (f32, f32, f32) {
    let (_, _, l_s) = rgb_to_hsl(r_s, g_s, b_s);
    let (h_b, s_b, _) = rgb_to_hsl(r_b, g_b, b_b);

    hsl_to_rgb(h_b, s_b, l_s)
}

/// https://www.w3.org/TR/compositing-1/#generalformula
///
/// `Cs = (1 - αb) x Cs + αb x B(Cb, Cs)`
/// `cs = Cs x αs`
/// `cb = Cb x αb`
/// `co = cs + cb x (1 - αs)`
/// Where
///  - Cs: is the source color
///  - Cb: is the backdrop color
///  - αs: is the source alpha
///  - αb: is the backdrop alpha
///  - B(Cb, Cs): is the mixing function
///
/// *The backdrop is the content behind the element and is what the element is composited with. This means that the backdrop is the result of compositing all previous elements.
fn composite(
    color_s: f32,
    alpha_s: f32,
    color_b: f32,
    alpha_b: f32,
    blend_f: &BlendFunction,
) -> f32 {
    let color_s = (1. - alpha_b) * color_s + alpha_b * blend_f(color_b, color_s);
    let cs = color_s * alpha_s;
    let cb = color_b * alpha_b;
    cs + cb * (1. - alpha_s)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_dodge() {
        let color_b = 1e-8;
        let color_s = 1.0;

        let result = color_dodge(color_b, color_s);

        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_color_burn() {
        let color_b = 0.9999999;
        let color_s = 0.5;

        let result = color_burn(color_b, color_s);

        assert_eq!(result, 1.0);
    }
}