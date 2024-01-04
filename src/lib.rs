//! Data structures and methods for working with PSD files.
//!
//! You are encouraged to read the PSD specification before contributing to this codebase.
//! This will help you better understand the current approach and discover ways to improve it.
//!
//! psd spec: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/

#![deny(missing_docs)]

use std::collections::HashMap;
use std::ops::Deref;

use thiserror::Error;

use sections::file_header_section::FileHeaderSectionError;
use sections::image_data_section::ImageDataSectionError;
use sections::image_resources_section::ImageResourcesSectionError;
use sections::layer_and_mask_information_section::layer::PsdLayerError;

use crate::psd_channel::IntoRgba;
pub use crate::psd_channel::{PsdChannelCompression, PsdChannelKind};
pub use crate::sections::file_header_section::{ColorMode, PsdDepth};
use crate::sections::image_data_section::ChannelBytes;
use crate::sections::image_data_section::ImageDataSection;
pub use crate::sections::image_resources_section::ImageResource;
use crate::sections::image_resources_section::ImageResourcesSection;
pub use crate::sections::image_resources_section::{DescriptorField, UnitFloatStructure};
pub use crate::sections::layer_and_mask_information_section::layer::PsdGroup;
pub use crate::sections::layer_and_mask_information_section::layer::PsdLayer;
use crate::sections::layer_and_mask_information_section::LayerAndMaskInformationSection;
use crate::sections::MajorSections;

use self::sections::file_header_section::FileHeaderSection;

mod blend;
mod psd_channel;
mod render;
mod sections;

/// An list of errors returned when processing PSD file.
///
/// This list is intended to grow over time and it is not recommended to exhaustively match against it.
#[derive(PartialEq, Debug, Error)]
#[non_exhaustive]
pub enum PsdError {
    /// Failed to parse PSD header
    #[error("Failed to parse PSD header: '{0}'.")]
    HeaderError(FileHeaderSectionError),
    /// Failed to parse PSD layer
    #[error("Failed to parse PSD layer: '{0}'.")]
    LayerError(PsdLayerError),
    /// Failed to parse PSD data section
    #[error("Failed to parse PSD data section: '{0}'.")]
    ImageError(ImageDataSectionError),
    /// Failed to parse PSD resource section
    #[error("Failed to parse PSD resource section: '{0}'.")]
    ResourceError(ImageResourcesSectionError),
}

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
    pub fn from_bytes(bytes: &[u8]) -> Result<Psd, PsdError> {
        let major_sections = MajorSections::from_bytes(bytes).map_err(PsdError::HeaderError)?;

        let file_header_section = FileHeaderSection::from_bytes(major_sections.file_header)
            .map_err(PsdError::HeaderError)?;

        let psd_width = file_header_section.width.0;
        let psd_height = file_header_section.height.0;
        let channel_count = file_header_section.channel_count.count();

        let layer_and_mask_information_section = LayerAndMaskInformationSection::from_bytes(
            major_sections.layer_and_mask,
            psd_width,
            psd_height,
        )
        .map_err(PsdError::LayerError)?;

        let image_data_section = ImageDataSection::from_bytes(
            major_sections.image_data,
            file_header_section.depth,
            psd_height,
            channel_count,
        )
        .map_err(PsdError::ImageError)?;

        let image_resources_section =
            ImageResourcesSection::from_bytes(major_sections.image_resources)
                .map_err(PsdError::ResourceError)?;

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
        &self.layer_and_mask_information_section.layers
    }

    /// Get a layer by name
    pub fn layer_by_name(&self, name: &str) -> Option<&PsdLayer> {
        self.layer_and_mask_information_section
            .layers
            .item_by_name(name)
    }

    /// Get a layer by index.
    ///
    /// index 0 is the bottom layer, index 1 is the layer above that, etc
    pub fn layer_by_idx(&self, idx: usize) -> &PsdLayer {
        self.layer_and_mask_information_section
            .layers
            .get(idx)
            .unwrap()
    }

    /// Get all of the groups in the PSD, in the order that they appear in the PSD file.
    pub fn groups(&self) -> &HashMap<u32, PsdGroup> {
        &self.layer_and_mask_information_section.groups
    }

    /// Get the group ID's in the order that they appear in Photoshop.
    /// (i.e. from the bottom of layers view to the top of the layers view).
    pub fn group_ids_in_order(&self) -> &Vec<u32> {
        self.layer_and_mask_information_section
            .groups
            .group_ids_in_order()
    }

    /// Returns sub layers of group by group id
    pub fn get_group_sub_layers(&self, id: &u32) -> Option<&[PsdLayer]> {
        match self.groups().get(id) {
            Some(group) => Some(
                &self.layer_and_mask_information_section.layers.deref()
                    [group.contained_layers.clone()],
            ),
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
    ) -> Result<Vec<u8>, PsdError> {
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
        let layers_to_flatten_top_down: Vec<&PsdLayer> = self
            .layers()
            .iter()
            .enumerate()
            // here we filter transparent layers and invisible layers
            .filter(|(_, layer)| (layer.opacity > 0 && layer.visible) || layer.clipping_mask)
            .filter(|(idx, layer)| filter((*idx, layer)))
            .map(|(_, layer)| layer)
            .collect();

        let pixel_count = self.width() * self.height();

        // If there aren't any layers left after filtering we return a complete transparent image.
        if layers_to_flatten_top_down.is_empty() {
            return Ok(vec![0; pixel_count as usize * 4]);
        }

        // During the process of flattening the PSD we might need to look at the pixels on one of
        // the layers below if an upper layer is transparent.
        //
        // Anytime we need to calculate the RGBA for a layer we cache it so that we don't need
        // to perform that operation again.
        let renderer = render::Renderer::new(&layers_to_flatten_top_down, self.width() as usize);

        let mut flattened_pixels = Vec::with_capacity((pixel_count * 4) as usize);

        // Iterate over each pixel and, if it is transparent, blend it with the pixel below it
        // recursively.
        for pixel_idx in 0..pixel_count as usize {
            let left = pixel_idx % self.width() as usize;
            let top = pixel_idx / self.width() as usize;
            let pixel_coord = (left, top);

            let blended_pixel = renderer.flattened_pixel(pixel_coord);

            flattened_pixels.push(blended_pixel[0]);
            flattened_pixels.push(blended_pixel[1]);
            flattened_pixels.push(blended_pixel[2]);
            flattened_pixels.push(blended_pixel[3]);
        }

        Ok(flattened_pixels)
    }
}

// Methods for working with the final flattened image data
impl Psd {
    /// Get the RGBA pixels for the PSD
    /// [ R,G,B,A, R,G,B,A, R,G,B,A, ...]
    pub fn rgba(&self) -> Vec<u8> {
        self.generate_rgba()
    }

    /// Get the compression level for the flattened image data
    pub fn compression(&self) -> &PsdChannelCompression {
        &self.image_data_section.compression
    }
}

// Methods for working with the image resources section
impl Psd {
    /// Resources from the image resources section of the PSD file
    pub fn resources(&self) -> &Vec<ImageResource> {
        &self.image_resources_section.resources
    }
}

impl IntoRgba for Psd {
    /// The PSD's final image is always the same size as the PSD so we don't need to transform
    /// indices like we do with layers.
    fn rgba_idx(&self, idx: usize) -> Option<usize> {
        Some(idx)
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

        assert_eq!(
            err,
            PsdError::HeaderError(FileHeaderSectionError::InvalidSignature {})
        );
    }
}
