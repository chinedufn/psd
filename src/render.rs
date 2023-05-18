use crate::blend;
use crate::sections::layer_and_mask_information_section::layer::BlendMode;
use crate::PsdLayer;
use std::cell::RefCell;
use std::iter::repeat_with;

pub(crate) struct Renderer<'a> {
    layers_to_flatten_top_down: &'a [&'a PsdLayer],
    cached_layer_rgba: Vec<RefCell<Option<Vec<u8>>>>,
    width: usize,
    pixel_cache: RefCell<Vec<(blend::Pixel, BlendMode)>>,
}

impl<'a> Renderer<'a> {
    pub(crate) fn new(
        layers_to_flatten_top_down: &'a [&'a PsdLayer],
        width: usize,
    ) -> Renderer<'a> {
        Renderer {
            layers_to_flatten_top_down: layers_to_flatten_top_down,
            cached_layer_rgba: repeat_with(|| RefCell::new(None))
                .take(layers_to_flatten_top_down.len())
                .collect(),
            width: width,
            pixel_cache: RefCell::new(Vec::with_capacity(layers_to_flatten_top_down.len())),
        }
    }

    fn pixel_rgba_for_layer(
        &'a self,
        flattened_layer_top_down_idx: usize,
        pixel_coord: (usize, usize),
    ) -> blend::Pixel {
        let layer = self.layers_to_flatten_top_down[flattened_layer_top_down_idx];

        // If we haven't already calculated the RGBA for this layer, calculate and cache it
        if self.cached_layer_rgba[flattened_layer_top_down_idx]
            .borrow()
            .is_none()
        {
            let pixels = layer.rgba();

            self.cached_layer_rgba[flattened_layer_top_down_idx].replace(Some(pixels));
        }

        let cached_layer_rgba = self.cached_layer_rgba[flattened_layer_top_down_idx].borrow();
        let layer_rgba = cached_layer_rgba.as_deref().unwrap();

        let (pixel_left, pixel_top) = pixel_coord;
        let pixel_idx = ((self.width * pixel_top) + pixel_left) * 4;

        let (start, end) = (pixel_idx, pixel_idx + 4);

        let pixel = &layer_rgba[start..end];
        let mut copy = [0; 4];
        copy.copy_from_slice(pixel);

        blend::apply_opacity(&mut copy, layer.opacity);
        copy
    }

    /// Get the pixel at a coordinate within this image.
    ///
    /// If that pixel has transparency, recursively blending it with the pixel
    /// below it until we reach a pixel with no transparency or the bottom of the stack.
    pub(crate) fn flattened_pixel(
        &'a self,
        // (left, top)
        pixel_coord: (usize, usize),
    ) -> [u8; 4] {
        let (pixel_left, pixel_top) = pixel_coord;
        let mut pixels = self.pixel_cache.borrow_mut();
        pixels.clear();
        for (idx, layer) in self.layers_to_flatten_top_down.iter().enumerate() {
            // If this pixel is out of bounds of this layer we return the pixel below it.
            // If there is no pixel below it we return a transparent pixel
            if (pixel_left as i32) < layer.layer_properties.layer_left
                || (pixel_left as i32) > layer.layer_properties.layer_right
                || (pixel_top as i32) < layer.layer_properties.layer_top
                || (pixel_top as i32) > layer.layer_properties.layer_bottom
            {
                continue;
            }

            let pixel = self.pixel_rgba_for_layer(idx, pixel_coord);
            pixels.push((pixel, layer.blend_mode));

            // This pixel is fully opaque, no point in going deeper
            if pixel[3] == 255 && layer.opacity == 255 {
                break;
            }
        }

        match pixels.pop() {
            Some((bottom_pixel, _)) => {
                pixels
                    .iter()
                    .rev()
                    .fold(bottom_pixel, |mut pixel_below, (pixel, blend_mode)| {
                        blend::blend_pixels(*pixel, pixel_below, *blend_mode, &mut pixel_below);

                        pixel_below
                    })
            }
            None => [0; 4],
        }
    }
}
