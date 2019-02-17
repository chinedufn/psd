use crate::sections::NotEnoughBytesError::LengthMarkerTooShort;
use failure::{Error, Fail};
use std::io::Cursor;
use std::io::Read;

/// The length of the entire file header section
const FILE_HEADER_SECTION_LEN: usize = 26;
/// The first four bytes of major sections (excluding File Header) indicate how long
/// the rest of the section is.
const LENGTH_MARKER_LEN: usize = 4;

pub(crate) mod file_header_section;
pub(crate) mod layer_and_mask_information_section;

/// References to the different major sections of a PSD file
pub struct MajorSections<'a> {
    pub(crate) file_header: &'a [u8],
    pub(crate) color_mode_data: &'a [u8],
    pub(crate) image_resources: &'a [u8],
    pub(crate) layer_and_mask: &'a [u8],
    pub(crate) image_data: &'a [u8],
}

/// The five kinds of major sections in a PSD file
#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]
pub enum MajorSectionKind {
    FileHeader,
    ColorModeData,
    ImageResources,
    LayerAndMask,
    ImageData,
}

impl<'a> MajorSections<'a> {
    /// Given the bytes of a PSD file, return the slices that correspond to each of the five
    /// major sections.
    ///
    /// ┌──────────────────┐
    /// │   File Header    │
    /// ├──────────────────┤
    /// │ Color Mode Data  │
    /// ├──────────────────┤
    /// │ Image Resources  │
    /// ├──────────────────┤
    /// │  Layer and Mask  │
    /// ├──────────────────┤
    /// │    Image Data    │
    /// └──────────────────┘
    ///
    /// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
    ///
    /// The Photoshop file format is divided into five major parts, as shown in the Photoshop
    /// file structure. The Photoshop file format has many length markers. Use these length markers
    /// to move from one section to the next. The length markers are usually padded with bytes to
    /// round to the nearest 2 or 4 byte interval.
    ///
    /// The file header has a fixed length; the other four sections are variable in length.
    ///
    /// When writing one of these sections, you should write all fields in the section, as Photoshop may try to read the entire section. Whenever writing a file and skipping bytes, you should explicitly write zeros for the skipped fields.
    ///
    /// When reading one of the length-delimited sections, use the length field to decide when you should stop reading. In most cases, the length field indicates the number of bytes, not records, following.
    ///
    /// The values in "Length" column in all tables are in bytes.
    ///
    /// All values defined as Unicode string consist of:
    ///
    /// A 4-byte length field, representing the number of characters in the string (not bytes).
    ///
    /// The string of Unicode values, two bytes per character.
    pub fn from_bytes(bytes: &[u8]) -> Result<MajorSections, Error> {
        let mut cursor = Cursor::new(bytes);;

        // File header section must be 26 bytes long.
        if bytes.len() < FILE_HEADER_SECTION_LEN {
            return Err(NotEnoughBytesError::FileHeader {
                total_bytes: bytes.len(),
            })?;
        }

        let file_header = &bytes[cursor.position() as usize..FILE_HEADER_SECTION_LEN];

        cursor.set_position(FILE_HEADER_SECTION_LEN as u64);

        // NOTE: The order matters here since we're mutating our cursor as we parse.
        // This is the same section ordering that PSD files use.
        let color_mode_data =
            get_major_section(&MajorSectionKind::ColorModeData, bytes, &mut cursor)?;
        let image_resources =
            get_major_section(&MajorSectionKind::ImageResources, bytes, &mut cursor)?;
        let layer_and_mask =
            get_major_section(&MajorSectionKind::LayerAndMask, bytes, &mut cursor)?;

        // The remaining bytes are the image data section.
        let image_data = &bytes[cursor.position() as usize..];

        Ok(MajorSections {
            file_header,
            color_mode_data,
            image_resources,
            layer_and_mask,
            image_data,
        })
    }
}

/// Get the bytes for a major section, including the section's length marker
/// (this will be the first four bytes of the returned byte slice)
///
/// We'll advance the cursor before returning the section bytes so that our major_sections
/// function can continue parsing without worrying about advancing the cursor.
fn get_major_section<'a>(
    section: &MajorSectionKind,
    bytes: &'a [u8],
    cursor: &mut Cursor<&[u8]>,
) -> Result<&'a [u8], Error> {
    check_length_marker_present(&section, bytes, &cursor)?;

    let data_len = get_major_section_data_len(&section, bytes, &cursor)?;

    let start = cursor.position() as usize;
    let end = start + LENGTH_MARKER_LEN + data_len;

    let data = &bytes[start..end];

    cursor.set_position(cursor.position() + data.len() as u64);

    Ok(data)
}

/// The length marker of a major section is 4 bytes long. I we see less than 4 bytes we'll
/// return an error indicating that we ran out of bytes before getting to the fourth byte of
/// the length marker.
fn check_length_marker_present(
    section: &MajorSectionKind,
    bytes: &[u8],
    cursor: &Cursor<&[u8]>,
) -> Result<(), Error> {
    // Color mode section must be at least four bytes
    let remaining_to_parse = bytes.len() - cursor.position() as usize;
    if remaining_to_parse < LENGTH_MARKER_LEN {
        return Err(NotEnoughBytesError::LengthMarkerTooShort {
            section: MajorSectionKind::FileHeader,
            len_marker_size: remaining_to_parse,
        })?;
    }

    Ok(())
}

/// The first four bytes of a major section (excluding the file header section) encodes a u32.
/// This u32 is the length of the rest of the data in the section.
///
/// We'll decode this u32 and return it as a usize
fn get_major_section_data_len(
    section: &MajorSectionKind,
    bytes: &[u8],
    cursor: &Cursor<&[u8]>,
) -> Result<usize, Error> {
    let mut four_bytes = [0; LENGTH_MARKER_LEN];

    let position = cursor.position() as usize;
    four_bytes.copy_from_slice(&bytes[position..position + 4]);

    let data_len = as_u32_be(&four_bytes) as usize;

    if cursor.position() as usize + data_len > bytes.len() {
        let actual_bytes_remaining = bytes.len() - cursor.position() as usize;
        return Err(NotEnoughBytesError::VariableSectionTooShort {
            section: *section,
            expected_byte_count: data_len,
            actual_bytes_remaining,
        })?;
    }

    Ok(data_len)
}

/// A section specified that it had more bytes than were provided.
///
/// For example, the FileHeaderSection requires 26 bytes, so if we only see
/// 25 bytes we'll return an error.
#[derive(Debug, Fail)]
pub enum NotEnoughBytesError {
    #[fail(
        display = r#"Could not parse the file header section.
    The file header section is comprised of the first 26 bytes (indices 0-25)
    of a PSD file, but only {} total bytes were provided."#,
        total_bytes
    )]
    FileHeader { total_bytes: usize },
    #[fail(
        display = r#"The {:#?} mode section must have a length marker that is 4 bytes long.
        Only {} bytes were found.
    "#,
        section, len_marker_size
    )]
    LengthMarkerTooShort {
        section: MajorSectionKind,
        len_marker_size: usize,
    },
    #[fail(
        display = r#"The {:#?} section's length section said that there were {} bytes of variable data,
    but only found {} bytes."#,
        section, expected_byte_count, actual_bytes_remaining
    )]
    VariableSectionTooShort {
        section: MajorSectionKind,
        expected_byte_count: usize,
        actual_bytes_remaining: usize,
    },
}

/// Convert a big endian byte array into a u32
pub(self) fn as_u32_be(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + ((array[3] as u32) << 0)
}

/// Convert a big endian byte array into a u16
pub(self) fn as_u16_be(array: &[u8; 2]) -> u16 {
    ((array[1] as u16) << 8) + ((array[0] as u16) << 0)
}

/// A Cursor wrapping bytes from a PSD file.
///
/// Provides methods that abstract common ways of parsing PSD bytes.
pub(self) struct PsdCursor<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> PsdCursor<'a> {
    /// Create a new PsdCursor
    pub fn new(bytes: &[u8]) -> PsdCursor {
        PsdCursor {
            cursor: Cursor::new(bytes),
        }
    }
}
