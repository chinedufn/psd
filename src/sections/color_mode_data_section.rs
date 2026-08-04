use std::io::{Seek, Write};
use thiserror::Error;
use super::{PsdBuffer, PsdCursor, PsdSerialize};

#[derive(Debug, PartialEq, Error)]
pub enum ColorModeDataSectionError {
    #[error("Unsupported color mode data")]
    UnsupportedColorMode,
}

#[derive(Debug, PartialEq)]
pub struct ColorModeDataSection {
    // NOTE: placeholder for actual color mode data
    color_mode_data: Vec<u8>,
}

impl ColorModeDataSection {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ColorModeDataSectionError> {
        // The color mode data section is a length-delimited block in the PSD format.
        // In most call sites (e.g. via `MajorSections::from_bytes`) the `bytes` slice includes the
        // 4-byte length prefix followed by the payload.
        //
        // For backwards compatibility with any internal call sites that may have passed only the
        // payload, we accept both forms.
        let color_mode_data = if bytes.len() >= 4 {
            let mut cursor = PsdCursor::new(bytes);
            let len = cursor.read_u32() as usize;
            let remaining = bytes.len().saturating_sub(4);

            if len <= remaining {
                cursor.read(len as u32).to_vec()
            } else {
                // Fallback: treat the entire input as opaque payload.
                bytes.to_vec()
            }
        } else {
            bytes.to_vec()
        };
        Ok(Self { color_mode_data })
    }

    pub fn new(color_mode_data: Vec<u8>) -> Self {
        Self { color_mode_data }
    }
}

impl PsdSerialize for ColorModeDataSection {
    fn write<T>(&self, buffer: &mut PsdBuffer<T>)
    where
        T: Write + Seek,
    {
        buffer.write_sized(|buf| buf.write(&self.color_mode_data));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that an empty color mode data payload round-trips through `write()` and `from_bytes()`.
    #[test]
    fn write_read_round_trip_empty_vec() {
        let initial = make_empty_section();
        let mut bytes: Vec<u8> = Vec::new();
        let mut buffer = PsdBuffer::new(&mut bytes);

        initial.write(&mut buffer);

        let result = ColorModeDataSection::from_bytes(&bytes).unwrap();
        assert_eq!(initial, result);
    }

    /// Verify that a non-empty color mode data payload round-trips through `write()` and `from_bytes()`.
    #[test]
    fn write_read_round_trip_non_empty_vec() {
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let initial = make_non_empty_section(data);
        let mut bytes: Vec<u8> = Vec::new();
        let mut buffer = PsdBuffer::new(&mut bytes);

        initial.write(&mut buffer);

        let result = ColorModeDataSection::from_bytes(&bytes).unwrap();
        assert_eq!(initial, result);
    }

    fn make_non_empty_section(input_vec: Vec<u8>) -> ColorModeDataSection {
        ColorModeDataSection::new(input_vec)
    }

    fn make_empty_section() -> ColorModeDataSection {
        let new_vec: Vec<u8> = Vec::new();
        ColorModeDataSection::new(new_vec)
    }
}
