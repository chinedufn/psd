use std::io::{Seek, Write};
use thiserror::Error;
use super::{PsdBuffer, PsdSerialize};

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
        // TODO: review this implementation
        // NOTE: process bytes if there's color mode data like Indexed or Duotone modes...
        let color_mode_data = bytes.to_vec();
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

    #[test]
    fn write_read_round_trip_empty_vec() {
        let initial = make_empty_section();
        let mut bytes: Vec<u8> = Vec::new();
        let bytes2: Vec<u8> = Vec::new();
        let mut buffer = PsdBuffer::new(&mut bytes);

        initial.write(&mut buffer);

        let result = ColorModeDataSection::from_bytes(&bytes2).unwrap();
        assert_eq!(initial, result);
    }

    #[test]
    fn write_read_round_trip_non_empty_vec() {
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let data_2: Vec<u8> = data.clone();
        let initial = make_non_empty_section(data);
        let mut bytes: Vec<u8> = Vec::new();
        let mut buffer = PsdBuffer::new(&mut bytes);

        initial.write(&mut buffer);

        let result = ColorModeDataSection::from_bytes(&data_2).unwrap();
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
