use failure::{Error, Fail};

use crate::sections::PsdCursor;

const EXPECTED_RESOURCE_BLOCK_SIGNATURE: [u8; 4] = [56, 66, 73, 77];
const RESOURCE_SLICES_INFO: i16 = 1050;

struct ImageResourcesBlock {
    resource_id: i16,
    name: String,
    data_range: (u64, u64)
}

#[derive(Debug)]
pub struct ImageResourcesSection {}

/// Represents an malformed resource block
#[derive(Debug, PartialEq, Fail)]
pub enum ImageResourcesSectionError {
    #[fail(
    display = r#"The first four bytes (indices 0-3) must always equal [56, 66, 73, 77],
         which in string form is '8BIM'."#
    )]
    InvalidSignature {},
}


impl ImageResourcesSection {
    pub fn from_bytes(bytes: &[u8]) -> Result<ImageResourcesSection, Error> {
        let mut cursor = PsdCursor::new(bytes);

        let length = cursor.read_u32()? as u64;
        let mut read = 0;
        while read < length {
            let pair = ImageResourcesSection::read_resource_block(&mut cursor)?;
            read = pair.0;

            let block = pair.1;
            match block.resource_id {
                RESOURCE_SLICES_INFO => {
                    let bytes = &cursor.get_ref()[block.data_range.0 as usize..block.data_range.1 as usize];
                    let mut cursor = PsdCursor::new(bytes);

                    let version = cursor.read_i32()?;
                    if version != 6 {
                        unimplemented!("Adobe Photoshop 6.0+ slice currently unsupported");
                    }

                    // We do not currently parse top of all the slices, skip it
                    cursor.read_i32()?;
                    // We do not currently parse left of all the slices, skip it
                    cursor.read_i32()?;
                    // We do not currently parse bottom of all the slices, skip it
                    cursor.read_i32()?;
                    // We do not currently parse right of all the slices, skip it
                    cursor.read_i32()?;
                    // We do not currently parse name of group of slices, skip it
                    cursor.read_unicode_string()?;

                    let number_of_slices = cursor.read_u32()?;
                    for n in 0..number_of_slices {
                        ImageResourcesSection::read_slice_block(&mut cursor)?;
                    }
                }
                _ => {}
            }
        }

        Ok(ImageResourcesSection {})
    }

    fn read_slice_block(cursor: &mut PsdCursor) -> Result<(), Error> {
        let id = cursor.read_u32()?;
        let group_id = cursor.read_i32()?;
        let origin = cursor.read_i32()?;

        // if origin = 1, Associated Layer ID is present
        if origin == 1 {
            cursor.read_i32()?;
        }

        // We do not currently parse name of group of slices, skip it
        cursor.read_unicode_string()?;
        // We do not currently parse type, skip it
        cursor.read_i32()?;
        // We do not currently parse top, skip it
        cursor.read_i32()?;
        // We do not currently parse left, skip it
        cursor.read_i32()?;
        // We do not currently parse bottom, skip it
        cursor.read_i32()?;
        // We do not currently parse right, skip it
        cursor.read_i32()?;
        // We do not currently parse URL, skip it
        cursor.read_unicode_string()?;
        // We do not currently parse target, skip it
        cursor.read_unicode_string()?;
        // We do not currently parse message skip it
        cursor.read_unicode_string()?;
        // We do not currently parse name of group of slices, skip it
        cursor.read_unicode_string()?;
        // We do not currently parse cell text HTML flag, skip it
        cursor.read_1()?;
        // We do not currently parse cell text, skip it
        cursor.read_unicode_string()?;
        // We do not currently parse horizontal alignment, skip it
        cursor.read_i32()?;
        // We do not currently parse vertical alignment, skip it
        cursor.read_i32()?;
        // We do not currently parse color, skip it
        // Note: in docs color is ARGB tuple
        cursor.read_i32()?;

        let descriptor_version = cursor.read_i32()?;
        ImageResourcesSection::read_descriptor_structure(cursor)?;

        Ok(())
    }

    fn read_resource_block(cursor: &mut PsdCursor) -> Result<(u64, ImageResourcesBlock), Error> {
        // First four bytes must be '8BIM'
        let signature = cursor.read_4()?;
        if signature != EXPECTED_RESOURCE_BLOCK_SIGNATURE {
            return Err(ImageResourcesSectionError::InvalidSignature {}.into());
        }

        let resource_id = cursor.read_i16()?;
        let name = cursor.read_pascal_string()?;

        let data_len = cursor.read_u32()?;
        let pos = cursor.position();
        // Note: data length is padded to even.
        let data_len = data_len + data_len % 2;
        let data_range = (pos, pos + data_len as u64);
        cursor.read(data_len)?;

        Ok((
            cursor.position(),
            ImageResourcesBlock {
                resource_id,
                name,
                data_range,
            },
        ))
    }

    fn read_descriptor_structure(cursor: &mut PsdCursor) -> Result<(), Error> {
        // TODO: make descriptor decoder
        Ok(())
    }
}
