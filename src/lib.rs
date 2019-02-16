//! Data structures and methods for working with PSD files.
//!
//! psd spec: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/

#![deny(missing_docs)]

mod file_header_section;
use failure::Error;

use self::file_header_section::FileHeaderSection;

pub use crate::file_header_section::ColorMode;

/// Represents the contents of a PSD file
///
/// ## PSB Support
///
/// We do not currently support PSB since the originally authors didn't need it, but adding
/// support should be trivial. If you'd like to support PSB please open an issue.
#[derive(Debug)]
pub struct Psd {
    file_section_header: FileHeaderSection,
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
        Ok(Psd {
            file_section_header: FileHeaderSection::from_bytes(&bytes[0..26])?,
        })
    }
}

// Information from the file section header
impl Psd {
    /// The width of the PSD file
    pub fn width(&self) -> u32 {
        self.file_section_header.width.0
    }

    /// The height of the PSD file
    pub fn height(&self) -> u32 {
        self.file_section_header.height.0
    }

    /// The number of bits per channel
    pub fn depth(&self) -> u8 {
        self.file_section_header.depth as u8
    }

    /// The color mode of the file
    pub fn color_mode(&self) -> ColorMode {
        self.file_section_header.color_mode
    }
}
