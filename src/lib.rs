//! Data structures and methods for working with PSD files.
//!
//! psd spec: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/

#![deny(missing_docs)]

pub use crate::sections::file_header_section::ColorMode;

use self::sections::file_header_section::FileHeaderSection;
use crate::sections::layer_and_mask_information_section::LayerAndMaskInformationSection;
use crate::sections::layer_and_mask_information_section::PsdLayer;
use crate::sections::MajorSections;
use failure::Error;
use std::collections::HashMap;

mod sections;

/// Represents the contents of a PSD file
///
/// ## PSB Support
///
/// We do not currently support PSB since the originally authors didn't need it, but adding
/// support should be trivial. If you'd like to support PSB please open an issue.
#[derive(Debug)]
pub struct Psd {
    file_header_section: FileHeaderSection,
    layer_and_mask_information_section: LayerAndMaskInformationSection,
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
        let layer_and_mask_information_section =
            LayerAndMaskInformationSection::from_bytes(major_sections.layer_and_mask)?;

        Ok(Psd {
            file_header_section,
            layer_and_mask_information_section,
        })
    }
}

// Information from the file section header
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

    /// Get all of the layers in the PSD
    pub fn layers(&self) -> &HashMap<String, PsdLayer> {
        &self.layer_and_mask_information_section.layers
    }
}
