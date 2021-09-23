use std::io::Cursor;
use anyhow::Result;
use thiserror::Error;
use crate::sections::file_header_section::{FileHeaderSectionError, EXPECTED_PSD_SIGNATURE};

/// The length of the entire file header section
const FILE_HEADER_SECTION_LEN: usize = 26;

pub mod file_header_section;
pub mod image_data_section;
pub mod image_resources_section;
pub mod layer_and_mask_information_section;

/// References to the different major sections of a PSD file
#[derive(Debug)]
pub struct MajorSections<'a> {
    pub(crate) file_header: &'a [u8],
    pub(crate) color_mode_data: &'a [u8],
    pub(crate) image_resources: &'a [u8],
    pub(crate) layer_and_mask: &'a [u8],
    pub(crate) image_data: &'a [u8],
}

impl<'a> MajorSections<'a> {
    /// Given the bytes of a PSD file, return the slices that correspond to each
    /// of the five major sections.
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
    pub fn from_bytes(bytes: &[u8]) -> Result<MajorSections> {
        // File header section must be 26 bytes long.
        if bytes.len() < FILE_HEADER_SECTION_LEN {
            return Err(NotEnoughBytesError::FileHeader {
                total_bytes: bytes.len(),
            }
            .into());
        }

        let mut cursor = PsdCursor::new(bytes);

        // First four bytes must be '8BPS'
        let signature = cursor.peek_4()?;
        if signature != EXPECTED_PSD_SIGNATURE {
            return Err(FileHeaderSectionError::InvalidSignature {}.into());
        }

        // File Header Section
        let file_header = &bytes[0..FILE_HEADER_SECTION_LEN];
        cursor.read(FILE_HEADER_SECTION_LEN as u32)?;

        let (color_start, color_end) = read_major_section_start_end(&mut cursor)?;
        let (img_res_start, img_res_end) = read_major_section_start_end(&mut cursor)?;
        let (layer_mask_start, layer_mask_end) = read_major_section_start_end(&mut cursor)?;

        // The remaining bytes are the image data section.
        let image_data = &bytes[cursor.position() as usize..];

        Ok(MajorSections {
            file_header,
            color_mode_data: &bytes[color_start..color_end],
            image_resources: &bytes[img_res_start..img_res_end],
            layer_and_mask: &bytes[layer_mask_start..layer_mask_end],
            image_data,
        })
    }
}

/// Get the start and end indices of a major section
fn read_major_section_start_end(cursor: &mut PsdCursor) -> Result<(usize, usize)> {
    let start = cursor.position() as usize;
    let data_len = cursor.read_u32()?;
    cursor.read(data_len)?;
    let end = cursor.position() as usize;

    Ok((start, end))
}

/// A section specified that it had more bytes than were provided.
///
/// For example, the FileHeaderSection requires 26 bytes, so if we only see
/// 25 bytes we'll return an error.
#[derive(Debug, Error)]
pub enum NotEnoughBytesError {
    #[error(
        r#"Could not parse the file header section.
    The file header section is comprised of the first 26 bytes (indices 0-25)
    of a PSD file, but only {total_bytes} total bytes were provided."#
    )]
    FileHeader { total_bytes: usize },
}

/// A Cursor wrapping bytes from a PSD file.
///
/// Provides methods that abstract common ways of parsing PSD bytes.
pub(crate) struct PsdCursor<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> PsdCursor<'a> {
    /// Create a new PsdCursor
    pub fn new(bytes: &[u8]) -> PsdCursor {
        PsdCursor {
            cursor: Cursor::new(bytes),
        }
    }

    /// Get the cursor's position
    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn seek(&mut self, pos: u64) {
        self.cursor.set_position(pos);
    }

    /// Get the underlying bytes in the cursor
    pub fn get_ref(&self) -> &[u8] {
        self.cursor.get_ref()
    }

    /// Advance the cursor by count bytes and return those bytes
    pub fn read(&mut self, count: u32) -> Result<&[u8]> {
        let start = self.cursor.position() as usize;
        let end = start + count as usize;
        let bytes = &self.cursor.get_ref()[start..end];

        self.cursor.set_position(end as u64);

        Ok(bytes)
    }

    pub fn peek_u32(&self) -> Result<u32> {
        let bytes = self.peek_4()?;
        Ok(u32_from_be_bytes(bytes))
    }

    /// Peek at the next four bytes
    pub fn peek_4(&self) -> Result<&[u8]> {
        self.peek(4)
    }

    /// Get the next n bytes without moving the cursor
    fn peek(&self, n: u8) -> Result<&[u8]> {
        let start = self.cursor.position() as usize;
        let end = start + n as usize;
        let bytes = &self.cursor.get_ref()[start..end];

        Ok(&bytes)
    }

    /// Read 1 byte
    pub fn read_1(&mut self) -> Result<&[u8]> {
        self.read(1)
    }

    /// Read 2 bytes
    pub fn read_2(&mut self) -> Result<&[u8]> {
        self.read(2)
    }

    /// Read 4 bytes
    pub fn read_4(&mut self) -> Result<&[u8]> {
        self.read(4)
    }

    /// Read 6 bytes
    pub fn read_6(&mut self) -> Result<&[u8]> {
        self.read(6)
    }

    /// Read 8 bytes
    pub fn read_8(&mut self) -> Result<&[u8]> {
        self.read(8)
    }

    /// Read 1 byte as a u8
    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_1()?[0])
    }

    /// Read 2 bytes as a u16
    pub fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_2()?;

        let mut array = [0; 2];
        array.copy_from_slice(bytes);

        Ok(u16::from_be_bytes(array))
    }

    /// Read 4 bytes as a u32
    pub fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_4()?;
        Ok(u32_from_be_bytes(bytes))
    }

    /// Read 1 byte as a i8
    pub fn read_i8(&mut self) -> Result<i8> {
        let bytes = self.read_1()?;

        let mut array = [0; 1];
        array.copy_from_slice(bytes);

        Ok(i8::from_be_bytes(array))
    }

    /// Read 2 bytes as a i16
    pub fn read_i16(&mut self) -> Result<i16> {
        let bytes = self.read_2()?;

        let mut array = [0; 2];
        array.copy_from_slice(bytes);

        Ok(i16::from_be_bytes(array))
    }

    /// Read 4 bytes as a i32
    pub fn read_i32(&mut self) -> Result<i32> {
        let bytes = self.read_4()?;

        let mut array = [0; 4];
        array.copy_from_slice(bytes);
        Ok(i32::from_be_bytes(array))
    }

    /// Read 8 bytes as a f64
    pub fn read_f64(&mut self) -> Result<f64> {
        let bytes = self.read_8()?;

        let mut array = [0; 8];
        array.copy_from_slice(bytes);

        Ok(f64::from_be_bytes(array))
    }

    /// Read 8 bytes as a i64
    pub fn read_i64(&mut self) -> Result<i64> {
        let bytes = self.read_8()?;

        let mut array = [0; 8];
        array.copy_from_slice(bytes);

        Ok(i64::from_be_bytes(array))
    }

    /// Reads 'Unicode string'
    ///
    /// Unicode string is
    /// A 4-byte length field, representing the number of UTF-16 code units in the string (not bytes).
    /// The string of Unicode values, two bytes per character and a two byte null for the end of the string.
    pub fn read_unicode_string(&mut self) -> Result<String> {
        self.read_unicode_string_padding(4)
    }

    /// Reads 'Unicode string' using specified padding
    ///
    /// Unicode string is
    /// A 4-byte length field, representing the number of UTF-16 code units in the string (not bytes).
    /// The string of Unicode values, two bytes per character and a two byte null for the end of the string.
    pub fn read_unicode_string_padding(&mut self, padding: usize) -> Result<String> {
        let length = self.read_u32()? as usize;
        // UTF-16 encoding - two bytes per character
        let length_bytes = length * 2;

        let data = self.read(length_bytes as u32)?;
        let result = String::from_utf16(&u8_slice_to_u16(data).as_slice()[..length as usize])?;

        self.read_padding(4 + length_bytes, padding)?;

        Ok(result)
    }

    fn read_padding(&mut self, size: usize, divisor: usize) -> Result<&[u8]> {
        let remainder = size % divisor;
        if remainder > 0 {
            let to_read = divisor - remainder;
            self.read(to_read as u32)
        } else {
            Ok(&[] as &[u8])
        }
    }

    /// Reads 'Pascal string'
    ///
    /// Pascal string is UTF-8 string, padded to make the size even
    /// (a null name consists of two bytes of 0)
    pub fn read_pascal_string(&mut self) -> Result<String> {
        let len = self.read_u8()?;
        let data = self.read(len as u32)?;
        let result = Ok(String::from_utf8(data.to_vec())?);

        // read null byte
        self.read_u8()?;
        result
    }
}

fn u8_slice_to_u16(bytes: &[u8]) -> Vec<u16> {
    return Vec::from(bytes)
        .chunks_exact(2)
        .into_iter()
        .map(|a| u16::from_be_bytes([a[0], a[1]]))
        .collect();
}

fn u32_from_be_bytes(bytes: &[u8]) -> u32 {
    let mut array = [0; 4];
    array.copy_from_slice(bytes);

    u32::from_be_bytes(array)
}
