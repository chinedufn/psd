use crate::PsdLayer;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::blend;

pub(crate) struct Renderer<'a> {
    layers_to_flatten_top_down: &'a [(usize, &'a PsdLayer)],
    cached_layer_rgba: RefCell<HashMap<usize, Vec<u8>>>,
    width: usize,
}

impl<'a> Renderer<'a> {
    pub(crate) fn new(
        layers_to_flatten_top_down: &'a [(usize, &'a PsdLayer)],
        width: usize,
    ) -> Renderer<'a> {
        Renderer {
            layers_to_flatten_top_down: layers_to_flatten_top_down,
            cached_layer_rgba: RefCell::new(HashMap::new()),
            width: width,
        }
    }

    fn pixel_rgba_for_layer(
        &'a self,
        flattened_layer_top_down_idx: usize,
        pixel_coord: (usize, usize),
    ) -> [u8; 4] {
        let layer = self.layers_to_flatten_top_down[flattened_layer_top_down_idx].1;

        let (pixel_left, pixel_top) = pixel_coord;

        // If we haven't already calculated the RGBA for this layer, calculate and cache it
        if self
            .cached_layer_rgba
            .borrow()
            .get(&flattened_layer_top_down_idx)
            .is_none()
        {
            let pixels = self.layers_to_flatten_top_down[flattened_layer_top_down_idx]
                .1
                .rgba();

            self.cached_layer_rgba
                .borrow_mut()
                .insert(flattened_layer_top_down_idx, pixels);
        }

        let cache = self.cached_layer_rgba.borrow();
        let layer_rgba = cache.get(&flattened_layer_top_down_idx).unwrap();

        let pixel_idx = ((self.width * pixel_top) + pixel_left) * 4;

        let (start, end) = (pixel_idx, pixel_idx + 4);

        let pixel = &layer_rgba[start..end];
        let mut copy = [0; 4];
        copy.copy_from_slice(pixel);

        blend::apply_opacity(&mut copy, layer.opacity);
        return copy;
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
        let mut pixels = vec![];
        for (idx, layer) in self.layers_to_flatten_top_down.iter().enumerate() {
            let layer = layer.1;

            // If this pixel is out of bounds of this layer we return the pixel below it.
            // If there is no pixel below it we return a transparent pixel
            if pixel_left < layer.layer_properties.layer_left as usize
                || pixel_left > layer.layer_properties.layer_right as usize
                || pixel_top < layer.layer_properties.layer_top as usize
                || pixel_top > layer.layer_properties.layer_bottom as usize
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

        if pixels.len() == 0 {
            return [0; 4];
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
