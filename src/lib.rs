//! Data structures and methods for working with PSD files.
//!
//! You are encouraged to read the PSD specification before contributing to this codebase.
//! This will help you better understand the current approach and discover ways to improve it.
//!
//! psd spec: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/

#![deny(missing_docs)]

use std::cell::RefCell;
use std::collections::HashMap;

use failure::Error;

pub use crate::psd_channel::{PsdChannelCompression, PsdChannelKind};
use crate::psd_channel::IntoRgba;
pub use crate::sections::file_header_section::{ColorMode, PsdDepth};
use crate::sections::image_data_section::ChannelBytes;
use crate::sections::image_data_section::ImageDataSection;
pub use crate::sections::image_resources_section::{DescriptorField, UnitFloatStructure};
use crate::sections::image_resources_section::{DescriptorStructure, ImageResourcesSection};
use crate::sections::layer_and_mask_information_section::layer::PsdGroup;
pub use crate::sections::layer_and_mask_information_section::layer::PsdLayer;
use crate::sections::layer_and_mask_information_section::LayerAndMaskInformationSection;
use crate::sections::MajorSections;

use self::sections::file_header_section::FileHeaderSection;

mod psd_channel;
mod sections;

/// Represents the contents of a PSD file
///
/// ## PSB Support
///
/// We do not currently support PSB since the original authors didn't need it, but adding
/// support should be trivial. If you'd like to support PSB please open an issue.
#[derive(Debug)]
pub struct Psd {
    file_header_section: FileHeaderSection,
    image_resources_section: ImageResourcesSection,
    layer_and_mask_information_section: LayerAndMaskInformationSection,
    image_data_section: ImageDataSection,
}

impl Psd {
    /// Create a Psd from a byte slice.
    ///
    /// You'll typically get these bytes from a PSD file.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let psd_bytes = include_bytes!("./my-psd-file.psd");
    ///
    /// let psd = Psd::from_bytes(psd_bytes);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Psd, Error> {
        let major_sections = MajorSections::from_bytes(bytes)?;

        let file_header_section = FileHeaderSection::from_bytes(major_sections.file_header)?;

        let psd_width = file_header_section.width.0;
        let psd_height = file_header_section.height.0;
        let channel_count = file_header_section.channel_count.count();

        let layer_and_mask_information_section = LayerAndMaskInformationSection::from_bytes(
            major_sections.layer_and_mask,
            psd_width,
            psd_height,
        )?;

        let image_data_section = ImageDataSection::from_bytes(
            major_sections.image_data,
            file_header_section.depth,
            psd_height,
            channel_count,
        )?;


        let image_resources_section = ImageResourcesSection::from_bytes(
            major_sections.image_resources
        )?;

        Ok(Psd {
            file_header_section,
            image_resources_section,
            layer_and_mask_information_section,
            image_data_section,
        })
    }
}

// Methods for working with the file section header
impl Psd {
    /// The width of the PSD file
    pub fn width(&self) -> u32 {
        self.file_header_section.width.0
    }

    /// The height of the PSD file
    pub fn height(&self) -> u32 {
        self.file_header_section.height.0
    }

    /// The number of bits per channel
    pub fn depth(&self) -> PsdDepth {
        self.file_header_section.depth
    }

    /// The color mode of the file
    pub fn color_mode(&self) -> ColorMode {
        self.file_header_section.color_mode
    }
}

// Methods for working with layers
impl Psd {
    /// Get all of the layers in the PSD
    pub fn layers(&self) -> &Vec<PsdLayer> {
        &self.layer_and_mask_information_section.layers.items()
    }

    /// Get a layer by name
    pub fn layer_by_name(&self, name: &str) -> Result<&PsdLayer, Error> {
        let item = self
            .layer_and_mask_information_section
            .layers
            .item_by_name(name)
            .unwrap();
        Ok(item)
    }

    /// Get a layer by index.
    ///
    /// index 0 is the bottom layer, index 1 is the layer above that, etc
    pub fn layer_by_idx(&self, idx: usize) -> Result<&PsdLayer, Error> {
        let item = self
            .layer_and_mask_information_section
            .layers
            .item_by_idx(idx)
            .unwrap();
        Ok(item)
    }

    /// Get a group by id.
    pub fn group_by_id(&self, id: usize) -> Option<&PsdGroup> {
        self.layer_and_mask_information_section
            .groups
            .item_by_idx(id)
    }

    /// Get a group by name
    pub fn group_by_name(&self, name: &str) -> Option<&PsdGroup> {
        self.layer_and_mask_information_section.groups.item_by_name(name)
    }

    /// Get all of the groups in the PSD
    pub fn groups(&self) -> &Vec<PsdGroup> {
        &self.layer_and_mask_information_section.groups.items()
    }

    /// Returns sub layers of group by group id
    pub fn get_sub_layers(&self, id: usize) -> Option<&[PsdLayer]> {
        match self.group_by_id(id) {
            Some(group) => {
                Some(&self.layer_and_mask_information_section.layers[&group.contained_layers])
            }
            None => None,
        }
    }

    /// Given a filter, combine all layers in the PSD that pass the filter into a vector
    /// of RGBA pixels.
    ///
    /// We'll start from the top most layer and iterate through the pixels.
    ///
    /// If the pixel is transparent, recursively blend it with the pixels below it until
    /// we hit an opaque pixel or we hit the bottom of the stack.
    ///
    /// TODO: Take the layer's blend mode into account when blending layers. Right now
    /// we just use ONE_MINUS_SRC_ALPHA blending regardless of the layer.
    pub fn flatten_layers_rgba(
        &self,
        filter: &dyn Fn((usize, &PsdLayer)) -> bool,
    ) -> Result<Vec<u8>, Error> {
        // When you create a PSD but don't create any new layers the bottom layer might not
        // show up in the layer and mask information section, so we won't see any layers.
        //
        // TODO: We should try and figure out where the layer name is so that we can return
        // a completely transparent image if it is filtered out. But this should be a rare
        // use case so we can just always return the final image for now.
        if self.layers().is_empty() {
            return Ok(self.rgba());
        }

        // Filter out layers based on the passed in filter.
        let mut layers_to_flatten_bottom_to_top: Vec<(usize, &PsdLayer)> = self
            .layers()
            .iter()
            .rev()
            .enumerate()
            .filter(|(idx, layer)| filter((*idx, layer)))
            .collect();
        layers_to_flatten_bottom_to_top.reverse();

        // index 0 = top layer ... index len = bottom layer
        let layers_to_flatten_top_to_bottom = layers_to_flatten_bottom_to_top;

        let pixel_count = self.width() * self.height();

        // If there aren't any layers left after filtering we return a complete transparent image.
        if layers_to_flatten_top_to_bottom.is_empty() {
            return Ok(vec![0; pixel_count as usize * 4]);
        }

        // During the process of flattening the PSD we might need to look at the pixels on one of
        // the layers below if an upper layer is transparent.
        //
        // Anytime we need to calculate the RGBA for a layer we cache it so that we don't need
        // to perform that operation again.
        let cached_layer_rgba = RefCell::new(HashMap::new());

        let mut flattened_pixels = Vec::with_capacity((pixel_count * 4) as usize);

        // Iterate over each pixel and, if it is transparent, blend it with the pixel below it
        // recursively.
        for pixel_idx in 0..pixel_count as usize {
            let left = pixel_idx % self.width() as usize;
            let top = pixel_idx / self.width() as usize;
            let pixel_coord = (left, top);

            let blended_pixel = self.flattened_pixel(
                0,
                pixel_coord,
                &layers_to_flatten_top_to_bottom,
                &cached_layer_rgba,
            );

            flattened_pixels.push(blended_pixel[0]);
            flattened_pixels.push(blended_pixel[1]);
            flattened_pixels.push(blended_pixel[2]);
            flattened_pixels.push(blended_pixel[3]);
        }

        Ok(flattened_pixels)
    }

    /// Get the pixel at a coordinate within this image.
    ///
    /// If that pixel has transparency, recursively blending it with the pixel
    /// below it until we reach a pixel with no transparency or the bottom of the stack.
    fn flattened_pixel(
        &self,
        // Top is 0, below that is 1, ... etc
        flattened_layer_top_down_idx: usize,
        // (left, top)
        pixel_coord: (usize, usize),
        layers_to_flatten_top_down: &[(usize, &PsdLayer)],
        cached_layer_rgba: &RefCell<HashMap<usize, Vec<u8>>>,
    ) -> [u8; 4] {
        let layer = layers_to_flatten_top_down[flattened_layer_top_down_idx].1;

        let (pixel_left, pixel_top) = pixel_coord;

        // If this pixel is out of bounds of this layer we return the pixel below it.
        // If there is no pixel below it we return a transparent pixel
        if pixel_left < layer.layer_properties.layer_left as usize
            || pixel_left > layer.layer_properties.layer_right as usize
            || pixel_top < layer.layer_properties.layer_top as usize
            || pixel_top > layer.layer_properties.layer_bottom as usize
        {
            if flattened_layer_top_down_idx + 1 < layers_to_flatten_top_down.len() {
                return self.flattened_pixel(
                    flattened_layer_top_down_idx + 1,
                    pixel_coord,
                    layers_to_flatten_top_down,
                    cached_layer_rgba,
                );
            } else {
                return [0; 4];
            }
        }

        // If we haven't already calculated the RGBA for this layer, calculate and cache it
        if cached_layer_rgba
            .borrow()
            .get(&flattened_layer_top_down_idx)
            .is_none()
        {
            let pixels = layers_to_flatten_top_down[flattened_layer_top_down_idx]
                .1
                .rgba()
                .unwrap();
            cached_layer_rgba
                .borrow_mut()
                .insert(flattened_layer_top_down_idx, pixels);
        }

        let pixel = {
            let cache = cached_layer_rgba.borrow();
            let layer_rgba = cache.get(&flattened_layer_top_down_idx).unwrap();

            let pixel_idx = ((self.width() as usize * pixel_top) + pixel_left) * 4;

            let (start, end) = (pixel_idx, pixel_idx + 4);

            let pixel = &layer_rgba[start..end];
            let mut copy = [0; 4];
            copy.copy_from_slice(pixel);
            copy
        };

        // This pixel is fully opaque, return it
        if pixel[3] == 255 {
            pixel
        } else {
            // If this pixel has some transparency, blend it with the layer below it

            let mut final_pixel = [0; 4];

            match flattened_layer_top_down_idx + 1 < layers_to_flatten_top_down.len() {
                // This pixel has some transparency and there is a pixel below it, blend them
                true => {
                    let pixel_below = self.flattened_pixel(
                        flattened_layer_top_down_idx + 1,
                        pixel_coord,
                        layers_to_flatten_top_down,
                        cached_layer_rgba,
                    );

                    blend_pixels(pixel, pixel_below, &mut final_pixel);
                    final_pixel
                }
                // There is no pixel below this layer, so use it even though it has transparency
                false => pixel,
            }
        }
    }
}

// blend the two pixels.
//
// TODO: Take the layer's blend mode into account when blending layers. Right now
// we just use ONE_MINUS_SRC_ALPHA blending regardless of the layer.
// Didn't bother cleaning this up to be readable since we need to replace it
// anyways. Need to blend based on the layer's blend mode.
fn blend_pixels(top: [u8; 4], bottom: [u8; 4], out: &mut [u8; 4]) {
    let top_alpha = top[3] as f32 / 255.;

    out[0] = (bottom[0] as f32 + ((top[0] as f32 - bottom[0] as f32) * top_alpha)) as u8;
    out[1] = (bottom[1] as f32 + ((top[1] as f32 - bottom[1] as f32) * top_alpha)) as u8;
    out[2] = (bottom[2] as f32 + ((top[2] as f32 - bottom[2] as f32) * top_alpha)) as u8;

    out[3] = 255;
}

// Methods for working with the final flattened image data
impl Psd {
    /// Get the RGBA pixels for the PSD
    /// [ R,G,B,A, R,G,B,A, R,G,B,A, ...]
    pub fn rgba(&self) -> Vec<u8> {
        self.generate_rgba().unwrap()
    }

    /// Get the compression level for the flattened image data
    pub fn compression(&self) -> &PsdChannelCompression {
        &self.image_data_section.compression
    }
}


// Methods for working with the image resources section
impl Psd {
    /// Get the metadata descriptors of the PSD
    pub fn descriptors(&self) -> Option<&Vec<DescriptorStructure>> {
        self.image_resources_section.descriptors.as_ref()
    }
}

impl IntoRgba for Psd {
    /// The PSD's final image is always the same size as the PSD so we don't need to transform
    /// indices like we do with layers.
    fn rgba_idx(&self, idx: usize) -> usize {
        idx
    }

    fn red(&self) -> &ChannelBytes {
        &self.image_data_section.red
    }

    fn green(&self) -> Option<&ChannelBytes> {
        match self.color_mode() {
            // For 16 bit grayscale images I'm sometimes seeing two channels.
            // Really not sure what the second channel is so until we know what it is we're ignoring it..
            ColorMode::Grayscale => None,
            _ => self.image_data_section.green.as_ref(),
        }
    }

    fn blue(&self) -> Option<&ChannelBytes> {
        self.image_data_section.blue.as_ref()
    }

    fn alpha(&self) -> Option<&ChannelBytes> {
        self.image_data_section.alpha.as_ref()
    }

    fn psd_width(&self) -> u32 {
        self.width()
    }

    fn psd_height(&self) -> u32 {
        self.height()
    }
}

#[cfg(test)]
mod tests {
    use crate::sections::file_header_section::FileHeaderSectionError;

    use super::*;

    // Makes sure non PSD files get caught right away before getting a chance to create problems
    #[test]
    fn psd_signature_fail() {
        let psd = include_bytes!("../tests/fixtures/green-1x1.png");

        let err = Psd::from_bytes(psd).expect_err("Psd::from_bytes() didn't catch the PNG file");
        let err = err.downcast_ref::<FileHeaderSectionError>();

        assert_eq!(err, Some(&FileHeaderSectionError::InvalidSignature {}));
    }
}
