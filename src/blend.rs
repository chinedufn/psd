use crate::sections::layer_and_mask_information_section::layer::BlendMode;

pub(crate) type Pixel = [u8; 4];

// Multiplies the pixel's current alpha by the passed in `opacity`
pub(crate) fn apply_opacity(pixel: &mut Pixel, opacity: u8) {
    let alpha = opacity as f32 / 255.;
    pixel[3] = (pixel[3] as f32 * alpha) as u8;
}

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
pub(crate) fn blend_pixels(top: Pixel, bottom: Pixel, blend_mode: BlendMode, out: &mut Pixel) {
    // TODO: make some optimizations
    let alpha_s = top[3] as f32 / 255.;
    let alpha_b = bottom[3] as f32 / 255.;
    let alpha_output = alpha_s + alpha_b * (1. - alpha_s);

    let (r_s, g_s, b_s) = (
        top[0] as f32 / 255.,
        top[1] as f32 / 255.,
        top[2] as f32 / 255.,
    );
    let (r_b, g_b, b_b) = (
        bottom[0] as f32 / 255.,
        bottom[1] as f32 / 255.,
        bottom[2] as f32 / 255.,
    );

    let blend_f = map_blend_mode(blend_mode);
    let (r, g, b) = (
        composite(r_s, alpha_s, r_b, alpha_b, blend_f) * 255.,
        composite(g_s, alpha_s, g_b, alpha_b, blend_f) * 255.,
        composite(b_s, alpha_s, b_b, alpha_b, blend_f) * 255.,
    );

    // NOTE: make all assignments _after_ all reads to avoid issues when top or bottom is out
    out[0] = (r.round() / alpha_output) as u8;
    out[1] = (g.round() / alpha_output) as u8;
    out[2] = (b.round() / alpha_output) as u8;
    out[3] = (255. * alpha_output).round() as u8;
}

type BlendFunction = dyn Fn(f32, f32) -> f32;

/// Returns blend function for given BlendMode
fn map_blend_mode(blend_mode: BlendMode) -> &'static BlendFunction {
    // Modes are sorted like in Photoshop UI
    // TODO: make other modes
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
        BlendMode::DarkerColor => &darker_color,
        // --------------------------------------
        BlendMode::Lighten => &lighten,
        BlendMode::Screen => &screen,
        BlendMode::ColorDodge => &color_dodge,
        BlendMode::LinearDodge => &linear_dodge,
        BlendMode::LighterColor => &lighter_color,
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
        BlendMode::Hue => &hue,
        BlendMode::Saturation => &saturation,
        BlendMode::Color => &color,
        BlendMode::Luminosity => &luminosity,
    }
}

fn pass_through(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
}

/// https://www.w3.org/TR/compositing-1/#blendingnormal
/// This is the default attribute which specifies no blending. The blending formula simply selects the source color.
///
/// `B(Cb, Cs) = Cs`
#[inline(always)]
fn normal(color_b: f32, color_s: f32) -> f32 {
    color_s
}

fn dissolve(color_b: f32, color_s: f32) -> f32 {
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
    if color_b == 1. {
        1.
    } else {
        (1. - (1. - color_s) / color_b).max(0.)
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
    (color_b - color_s - 1.).max(0.)
}

fn darker_color(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
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
    if color_b == 0. {
        0.
    } else if color_s == 1. {
        1.
    } else {
        (color_b / (1. - color_s)).min(1.)
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

fn lighter_color(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
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

fn vivid_light(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
}

fn linear_light(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
}

#[inline(always)]
fn pin_light(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
}

#[inline(always)]
fn hard_mix(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
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
    if color_s == 0. {
        color_b
    } else {
        color_b / color_s
    }
}

fn hue(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
}

fn saturation(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
}

fn color(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
}

fn luminosity(color_b: f32, color_s: f32) -> f32 {
    unimplemented!()
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
