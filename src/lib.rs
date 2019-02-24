//! Data structures and methods for working with PSD files.
//!
//! You are encouraged to read the PSD specification before contributing to this codebase.
//! This will help you better understand the current approach and discover ways to improve it.
//!
//! psd spec: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/

#![deny(missing_docs)]

pub use crate::sections::file_header_section::ColorMode;

use self::sections::file_header_section::FileHeaderSection;
use crate::psd_channel::InsertChannelBytes;
use crate::sections::image_data_section::ImageDataSection;
pub use crate::sections::layer_and_mask_information_section::layer::PsdLayer;
use crate::sections::layer_and_mask_information_section::LayerAndMaskInformationSection;
use crate::sections::MajorSections;
use failure::Error;
use std::cell::RefCell;
use std::collections::HashMap;
pub use crate::psd_channel::{PsdChannelCompression, PsdChannelKind};

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
            psd_width,
            psd_height,
            channel_count,
        )?;

        Ok(Psd {
            file_header_section,
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
    pub fn depth(&self) -> u8 {
        self.file_header_section.depth as u8
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
    pub fn layer_by_name(&self, name: &str) -> Result<&PsdLayer, Error> {
        let layer_idx = self
            .layer_and_mask_information_section
            .layer_names
            .get(name)
            .unwrap();
        Ok(&self.layer_and_mask_information_section.layers[*layer_idx])
    }

    /// Get a layer by index.
    ///
    /// index 0 is the bottom layer, index 1 is the layer above that, etc
    pub fn layer_by_idx(&self, idx: usize) -> Result<&PsdLayer, Error> {
        Ok(&self.layer_and_mask_information_section.layers[idx])
    }

    /// Given a filter, combine all layers in the PSD that pass the filter into a vector
    /// of RGBA pixels.
    pub fn flatten_layers_rgba(
        &self,
        filter: &Fn((usize, &PsdLayer)) -> bool,
    ) -> Result<Vec<u8>, Error> {
        // Start from the top layer so that if none of the pixels are transparent we don't
        // need to look at any of the layers below it.
        // If they are we only continue to look downwards until we no longer see transparent
        // pixels.
        let mut layers_to_flatten: Vec<(usize, &PsdLayer)> = self
            .layers()
            .iter()
            .enumerate()
            .filter(|(idx, layer)| filter((*idx, layer)))
            .collect();
        layers_to_flatten.reverse();

        let pixels = self.width() * self.height();

        if layers_to_flatten.len() == 0 {
            return Ok(vec![0; pixels as usize * 4]);
        }

        let mut cached_layer_rgba = RefCell::new(HashMap::new());

        let layer_count = layers_to_flatten.len();

        let mut flattened_pixels = Vec::with_capacity((self.width() * self.height() * 4) as usize);

        for pixel_idx in 0..pixels as usize {
            let left = pixel_idx % self.width() as usize;
            let top = pixel_idx / self.width() as usize;
            let pixel_coord = (left, top);

            let pixel =
                self.flattened_pixel(0, pixel_coord, &layers_to_flatten, &mut cached_layer_rgba);
            flattened_pixels.push(pixel[0]);
            flattened_pixels.push(pixel[1]);
            flattened_pixels.push(pixel[2]);
            flattened_pixels.push(pixel[3]);
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
        layers_to_flatten_top_down: &Vec<(usize, &PsdLayer)>,
        cached_layer_rgba: &RefCell<HashMap<usize, Vec<u8>>>,
    ) -> [u8; 4] {
        let layer = layers_to_flatten_top_down[flattened_layer_top_down_idx].1;

        let (left, top) = pixel_coord;

        // If this pixel is out of bounds of this layer we return the pixel below it.
        // If there is no pixel below it we return a transparent pixel
        if left < layer.layer_left as usize
            || left > layer.layer_right as usize
            || top < layer.layer_top as usize
            || top > layer.layer_bottom as usize
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

        let cache = cached_layer_rgba.borrow();
        let rgba = cache.get(&flattened_layer_top_down_idx).unwrap();

        // FIXME: Just pass the pixel index in
        let pixel_idx = ((self.width() as usize * top) + left) * 4;

        let (start, end) = (pixel_idx, pixel_idx + 4);
        let pixel = &rgba[start..end];

        // This pixel as no transparency, use it
        if pixel[3] == 255 {
            let mut final_pixel = [0; 4];
            final_pixel.copy_from_slice(&pixel);
            final_pixel
        } else {
            let mut final_pixel = [0; 4];

            if flattened_layer_top_down_idx + 1 < layers_to_flatten_top_down.len() {
                let pixel_below = self.flattened_pixel(
                    flattened_layer_top_down_idx + 1,
                    pixel_coord,
                    layers_to_flatten_top_down,
                    cached_layer_rgba,
                );

                // FIXME: Move into method and clean up
                // blend the two pixels.
                // ((thisColor * thisAlpha) + (otherColor * (1 - thisAlpha)) / 2);
                final_pixel[0] = (((pixel[0] as u16 * pixel[3] as u16)
                    + (pixel_below[0] as u16 * (255 - pixel[3] as u16)))
                    / 2) as u8;
                final_pixel[1] = (((pixel[1] as u16 * pixel[3] as u16)
                    + (pixel_below[1] as u16 * (255 - pixel[3] as u16)))
                    / 2) as u8;
                final_pixel[2] = (((pixel[2] as u16 * pixel[3] as u16)
                    + (pixel_below[2] as u16 * (255 - pixel[3] as u16)))
                    / 2) as u8;
                final_pixel[3] = 255;

                final_pixel
            } else {
                final_pixel.copy_from_slice(pixel);
                final_pixel
            }
        }
    }
}

// Methods for working with the final flattened image data
impl Psd {
    /// Get the RGBA pixels for the PSD
    /// [ R,G,B,A, R,G,B,A, R,G,B,A, ...]
    pub fn rgba(&self) -> Vec<u8> {
        let rgba_len = (self.width() * self.height() * 4) as usize;

        // We use 119 because it's a weird number so we can easily notice in case
        // we're ever parsing something incorrectly.
        let mut rgba = vec![119; rgba_len];

        use crate::psd_channel::PsdChannelKind::*;

        self.insert_channel_bytes(&mut rgba, &Red, &self.image_data_section.red);
        self.insert_channel_bytes(&mut rgba, &Green, &self.image_data_section.green);
        self.insert_channel_bytes(&mut rgba, &Blue, &self.image_data_section.blue);

        if let Some(alpha_channel) = &self.image_data_section.alpha {
            self.insert_channel_bytes(&mut rgba, &TransparencyMask, alpha_channel);
        } else {
            // If there is no transparency data then the image is opaque
            for idx in 0..rgba_len / 4 {
                rgba[idx * 4 + 3] = 255;
            }
        }

        rgba
    }

    /// Get the compression level for the flattened image data
    pub fn compression(&self) -> &PsdChannelCompression {
        &self.image_data_section.compression
    }
}

impl InsertChannelBytes for Psd {
    /// The PSD's final image is always the same size as the PSD so we don't need to transform
    /// indices like we do with layers.
    fn rgba_idx(&self, idx: usize) -> usize {
        idx
    }
}
