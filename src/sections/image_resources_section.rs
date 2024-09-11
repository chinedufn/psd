use std::io::{Seek, Write};
use std::ops::Range;

use thiserror::Error;

use crate::sections::image_resources_section::image_resource::descriptor_structure::{
    DescriptorStructure, ImageResourcesDescriptorError,
};
use crate::sections::PsdCursor;

use super::{PsdBuffer, PsdDeserialize, PsdSerialize};
pub use crate::sections::image_resources_section::image_resource::{
    ImageResource, SlicesImageResource,
};

const EXPECTED_RESOURCE_BLOCK_SIGNATURE: [u8; 4] = [56, 66, 73, 77];
const RESOURCE_SLICES_INFO: i16 = 1050;

pub mod image_resource;

#[allow(dead_code)]
struct ImageResourcesBlock {
    resource_id: i16,
    name: String,
    data_range: Range<usize>,
}

#[derive(Debug)]
pub struct ImageResourcesSection {
    pub(crate) resources: Vec<ImageResource>,
}

/// Represents an malformed resource block
#[derive(Debug, PartialEq, Error)]
pub enum ImageResourcesSectionError {
    #[error(
        r#"The first four bytes (indices 0-3) must always equal [56, 66, 73, 77],
         which in string form is '8BIM'."#
    )]
    InvalidSignature {},

    #[error("Invalid resource descriptor: {0}")]
    InvalidResource(ImageResourcesDescriptorError),
}

impl PsdSerialize for ImageResourcesSection {
    fn write<T>(&self, buffer: &mut PsdBuffer<T>)
    where
        T: Write + Seek,
    {
        buffer.write_sized(|buf| {
            for resource in self.resources.iter() {
                resource.write(buf)
            }
        });
    }
}

impl PsdDeserialize for ImageResourcesSection {
    type Error = ImageResourcesSectionError;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = PsdCursor::new(bytes);

        let mut resources = vec![];

        let length = cursor.read_u32() as u64;

        while cursor.position() < length {
            let block = ImageResourcesSection::read_resource_block(&mut cursor)?;

            let rid = block.resource_id;
            match rid {
                _ if rid == RESOURCE_SLICES_INFO => {
                    let slices_image_resource =
                        SlicesImageResource::from_bytes(&cursor.get_ref()[block.data_range])
                            .map_err(ImageResourcesSectionError::InvalidResource)?;
                    resources.push(ImageResource::Slices(slices_image_resource));
                }
                _ => {}
            }
        }

        assert_eq!(cursor.position(), length + 4);

        Ok(ImageResourcesSection { resources })
    }
}

impl ImageResourcesSection {
    /// +----------+--------------------------------------------------------------------------------------------------------------------+
    /// |  Length  |                                                    Description                                                     |
    /// +----------+--------------------------------------------------------------------------------------------------------------------+
    /// | 4        | Signature: '8BIM'                                                                                                  |
    /// | 2        | Unique identifier for the resource. Image resource IDs contains a list of resource IDs used by Photoshop.          |
    /// | Variable | Name: Pascal string, padded to make the size even (a null name consists of two bytes of 0)                         |
    /// | 4        | Actual size of resource data that follows                                                                          |
    /// | Variable | The resource data, described in the sections on the individual resource types. It is padded to make the size even. |
    /// +----------+--------------------------------------------------------------------------------------------------------------------+
    fn read_resource_block(
        cursor: &mut PsdCursor,
    ) -> Result<ImageResourcesBlock, ImageResourcesSectionError> {
        // First four bytes must be '8BIM'
        let signature = cursor.read_4();
        if signature != EXPECTED_RESOURCE_BLOCK_SIGNATURE {
            return Err(ImageResourcesSectionError::InvalidSignature {});
        }

        let resource_id = cursor.read_i16();
        let name = cursor.read_pascal_string();

        let data_len = cursor.read_u32();
        let pos = cursor.position() as usize;
        // Note: data length is padded to even.
        let data_len = data_len + data_len % 2;
        let data_range = Range {
            start: pos,
            end: pos + data_len as usize,
        };
        cursor.read(data_len);

        Ok(ImageResourcesBlock {
            resource_id,
            name,
            data_range,
        })
    }
}
