use std::convert::TryFrom;
use thiserror::Error;

use crate::sections::{image_resources_section::EXPECTED_RESOURCE_BLOCK_SIGNATURE, PsdSerialize};

pub mod descriptor_structure;
pub mod slices;

pub use slices::SlicesImageResource;

/// An image resource from the image resources section
#[derive(Debug)]
#[allow(missing_docs)]
pub enum ImageResource {
    Slices(SlicesImageResource),
}

impl PsdSerialize for ImageResource {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        buffer.write(EXPECTED_RESOURCE_BLOCK_SIGNATURE);

        let id: ImageResourceId = self.into();
        buffer.write(id.into_bytes());

        // TODO: do we need to write this or is it only when we are at an odd buffer index?
        //       reference: https://github.com/yu-icchi/go-psd/blob/5976db4c66f0/encode.go#L110
        buffer.write_pascal_string("");

        match self {
            Self::Slices(slices) => buffer.write_sized(|buf| buf.pad(2, |buf| slices.write(buf))),
        }
    }
}

enum ImageResourceId {
    Slices = 1050,
}

impl ImageResourceId {
    fn into_bytes(self) -> [u8; 2] {
        (self as i16).to_be_bytes()
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum ImageResourceIdError {
    #[error("Invalid resource id: {0}")]
    InvalidResourceId(i16),
}
impl TryFrom<i16> for ImageResourceId {
    type Error = ImageResourceIdError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            x if x == ImageResourceId::Slices as i16 => Ok(ImageResourceId::Slices),
            _ => Err(ImageResourceIdError::InvalidResourceId(value)),
        }
    }
}

impl From<&ImageResource> for ImageResourceId {
    fn from(value: &ImageResource) -> Self {
        match value {
            ImageResource::Slices(_) => ImageResourceId::Slices,
        }
    }
}
