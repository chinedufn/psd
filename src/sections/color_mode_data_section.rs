use std::io::{Seek, Write};
use thiserror::Error;

use super::{PsdBuffer, PsdSerialize};

#[derive(Debug, PartialEq, Error)]
pub enum ColorModeDataSectionError {}

#[derive(Debug, PartialEq)]
pub struct ColorModeDataSection {}

impl ColorModeDataSection {
    pub fn from_bytes(_bytes: &[u8]) -> Result<Self, ColorModeDataSectionError> {
        Ok(Self {})
    }
}

impl PsdSerialize for ColorModeDataSection {
    fn write<T>(&self, buffer: &mut PsdBuffer<T>)
    where
        T: Write + Seek,
    {
        buffer.write_sized(|buf| buf.write([]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_round_trip() {
        let initial = make_section();
        let mut bytes: Vec<u8> = vec![];
        let mut buffer = PsdBuffer::new(&mut bytes);

        initial.write(&mut buffer);

        let result = ColorModeDataSection::from_bytes(&bytes).unwrap();
        assert_eq!(initial, result);
    }

    fn make_section() -> ColorModeDataSection {
        ColorModeDataSection {}
    }
}
