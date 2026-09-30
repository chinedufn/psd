use crate::psd_channel::{apply_prediction, zip_decompress, PsdChannelCompression};
use crate::sections::PsdCursor;
use crate::PsdDepth;
use thiserror::Error;

/// Represents an malformed image data
#[derive(Debug, PartialEq, Error)]
pub enum ImageDataSectionError {
    #[error(
        r#"Only 8 and 16 bit depths are supported at the moment.
    If you'd like to see 1 and 32 bit depths supported - please open an issue."#
    )]
    UnsupportedDepth,

    #[error("{compression} is an invalid layer channel compression. Must be 0, 1, 2 or 3")]
    InvalidCompression { compression: u16 },
}

/// The ImageDataSection comes from the final section in the PSD that contains the pixel data
/// of the final PSD image (the one that comes from combining all of the layers).
///
/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// The last section of a Photoshop file contains the image pixel data.
/// Image data is stored in planar order: first all the red data, then all the green data, etc.
/// Each plane is stored in scan-line order, with no pad bytes,
///
/// | Length   | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
/// |----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | 2        | Compression method: <br> 0 = Raw image data <br> 1 = RLE compressed the image data starts with the byte counts for all the scan lines (rows * channels), with each count stored as a two-byte value. The RLE compressed data follows, with each scan line compressed separately. The RLE compression is the same compression algorithm used by the Macintosh ROM routine PackBits , and the TIFF standard. <br> 2 = ZIP without prediction <br> 3 = ZIP with prediction. |
/// | Variable | The image data. Planar order = RRR GGG BBB, etc.                                                                                                                                                                                                                                                                                                                                                                                                                         |
#[derive(Debug)]
pub struct ImageDataSection {
    /// The compression method for the image.
    pub(crate) compression: PsdChannelCompression,
    /// The red channel of the final image
    pub(crate) red: ChannelBytes,
    /// The green channel of the final image
    pub(crate) green: Option<ChannelBytes>,
    /// the blue channel of the final image
    pub(crate) blue: Option<ChannelBytes>,
    /// the alpha channel of the final image.
    /// If there is no alpha channel then it is a fully opaque image.
    pub(crate) alpha: Option<ChannelBytes>,
}

impl ImageDataSection {
    /// Create an ImageDataSection from the bytes in the corresponding section in a PSD file
    /// (including the length market)
    pub fn from_bytes(
        bytes: &[u8],
        depth: PsdDepth,
        psd_width: u32,
        psd_height: u32,
        channel_count: u8,
    ) -> Result<ImageDataSection, ImageDataSectionError> {
        let mut _cursor = PsdCursor::new(bytes);
        let channel_count = channel_count as usize;

        let mut cursor = PsdCursor::new(bytes);
        let compression = cursor.read_u16();
        let compression = PsdChannelCompression::new(compression)
            .ok_or(ImageDataSectionError::InvalidCompression { compression })?;

        let (red, green, blue, alpha) = match compression {
            PsdChannelCompression::RawData => {
                // First 2 bytes were compression bytes
                let channel_bytes = &bytes[2..];
                let channel_byte_count = channel_bytes.len();

                let bytes_per_channel = channel_byte_count / channel_count;

                // First bytes are red
                let mut red: Vec<u8> = channel_bytes[..bytes_per_channel].into();

                // Next bytes are green
                let green = if channel_count >= 2 {
                    Some(ChannelBytes::RawData(
                        channel_bytes[bytes_per_channel..2 * bytes_per_channel].into(),
                    ))
                } else {
                    None
                };

                // Then comes blue
                let blue = if channel_count >= 3 {
                    Some(ChannelBytes::RawData(
                        channel_bytes[2 * bytes_per_channel..3 * bytes_per_channel].into(),
                    ))
                } else {
                    None
                };

                // And optionally alpha bytes
                let alpha = if channel_count == 4 {
                    Some(ChannelBytes::RawData(
                        channel_bytes[3 * bytes_per_channel..4 * bytes_per_channel].to_vec(),
                    ))
                } else {
                    None
                };

                match depth {
                    PsdDepth::Eight => (ChannelBytes::RawData(red), green, blue, alpha),
                    // If this is a 16bit image there will be two bytes per pixel. We
                    // currently only support one byte per pixel so we convert the 2 bytes
                    // back down into 1 byte by mapping 0-65535 down to 0-255
                    PsdDepth::Sixteen => {
                        for idx in 0..red.len() / 2 {
                            let bytes = [red[2 * idx], red[2 * idx + 1]];
                            let bits16 = u16::from_be_bytes(bytes);
                            red[idx] = (bits16 / 256) as u8;
                        }
                        red.truncate(red.len() / 2);

                        (ChannelBytes::RawData(red), green, blue, alpha)
                    }
                    _ => return Err(ImageDataSectionError::UnsupportedDepth),
                }
            }
            // # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
            //
            // RLE compressed the image data starts with the byte counts for all the scan lines
            // (rows * channels), with each count stored as a two-byte value. The RLE compressed
            // data follows, with each scan line compressed separately. The RLE compression is
            // the same compression algorithm used by the Macintosh ROM routine PackBits,
            // and the TIFF standard.
            PsdChannelCompression::RleCompressed => {
                let mut red_byte_count = 0;
                let mut green_byte_count = if channel_count >= 2 { Some(0) } else { None };
                let mut blue_byte_count = if channel_count >= 3 { Some(0) } else { None };
                let mut alpha_byte_count = if channel_count == 4 { Some(0) } else { None };

                for _ in 0..psd_height {
                    red_byte_count += cursor.read_u16() as usize;
                }

                if let Some(ref mut green_byte_count) = green_byte_count {
                    for _ in 0..psd_height {
                        *green_byte_count += cursor.read_u16() as usize;
                    }
                }

                if let Some(ref mut blue_byte_count) = blue_byte_count {
                    for _ in 0..psd_height {
                        *blue_byte_count += cursor.read_u16() as usize;
                    }
                }

                if let Some(ref mut alpha_byte_count) = alpha_byte_count {
                    for _ in 0..psd_height {
                        *alpha_byte_count += cursor.read_u16() as usize;
                    }
                }

                // 2 bytes for compression level, then 2 bytes for each scanline of each channel
                // We're skipping over the bytes that describe the length of each scanling since
                // we don't currently use them. We might re-think this in the future when we
                // implement serialization of a Psd back into bytes.. But not a concern at the
                // moment.
                let channel_data_start = 2 + (channel_count * psd_height as usize * 2);

                let (red_start, red_end) =
                    (channel_data_start, channel_data_start + red_byte_count);

                let red = bytes[red_start..red_end].into();

                let green = match green_byte_count {
                    Some(green_byte_count) => {
                        let green_start = red_end;
                        let green_end = green_start + green_byte_count;
                        Some(ChannelBytes::RleCompressed(
                            bytes[green_start..green_end].into(),
                        ))
                    }
                    None => None,
                };

                let blue = match blue_byte_count {
                    Some(blue_byte_count) => {
                        let blue_start = red_end + green_byte_count.unwrap();
                        let blue_end = blue_start + blue_byte_count;
                        Some(ChannelBytes::RleCompressed(
                            bytes[blue_start..blue_end].into(),
                        ))
                    }
                    None => None,
                };

                let alpha = match alpha_byte_count {
                    Some(alpha_byte_count) => {
                        let alpha_start =
                            red_end + green_byte_count.unwrap() + blue_byte_count.unwrap();
                        let alpha_end = alpha_start + alpha_byte_count;
                        Some(ChannelBytes::RleCompressed(
                            bytes[alpha_start..alpha_end].into(),
                        ))
                    }
                    None => None,
                };

                (ChannelBytes::RleCompressed(red), green, blue, alpha)
            }
            PsdChannelCompression::ZipWithoutPrediction | PsdChannelCompression::ZipWithPrediction => {
                let channel_data = &bytes[2..];
                let mut decompressed = zip_decompress(channel_data);

                if compression == PsdChannelCompression::ZipWithPrediction {
                    apply_prediction(&mut decompressed, psd_width as usize, psd_height as usize * channel_count, depth);
                }

                if depth == PsdDepth::Sixteen {
                    for idx in 0..decompressed.len() / 2 {
                        let bytes = [decompressed[2 * idx], decompressed[2 * idx + 1]];
                        let bits16 = u16::from_be_bytes(bytes);
                        decompressed[idx] = (bits16 / 256) as u8;
                    }
                    decompressed.truncate(decompressed.len() / 2);
                }

                let bytes_per_channel = decompressed.len() / channel_count;

                let red = decompressed[..bytes_per_channel].to_vec();

                let green = if channel_count >= 2 {
                    Some(ChannelBytes::RawData(
                        decompressed[bytes_per_channel..2 * bytes_per_channel].to_vec(),
                    ))
                } else {
                    None
                };

                let blue = if channel_count >= 3 {
                    Some(ChannelBytes::RawData(
                        decompressed[2 * bytes_per_channel..3 * bytes_per_channel].to_vec(),
                    ))
                } else {
                    None
                };

                let alpha = if channel_count == 4 {
                    Some(ChannelBytes::RawData(
                        decompressed[3 * bytes_per_channel..4 * bytes_per_channel].to_vec(),
                    ))
                } else {
                    None
                };

                (ChannelBytes::RawData(red), green, blue, alpha)
            }
        };

        Ok(ImageDataSection {
            compression,
            red,
            green,
            blue,
            alpha,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ChannelBytes {
    RawData(Vec<u8>),
    RleCompressed(Vec<u8>),
    ZipWithoutPrediction(Vec<u8>),
    ZipWithPrediction(Vec<u8>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn test_image_data_section_zip_without_prediction() {
        let width = 2;
        let height = 2;
        let channel_count = 3;
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120];

        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut bytes = vec![0, 2]; // ZIP without prediction
        bytes.extend_from_slice(&compressed);

        let section = ImageDataSection::from_bytes(
            &bytes,
            PsdDepth::Eight,
            width,
            height,
            channel_count as u8,
        )
        .unwrap();

        assert_eq!(
            section.compression,
            PsdChannelCompression::ZipWithoutPrediction
        );

        if let ChannelBytes::RawData(red) = section.red {
            assert_eq!(red, vec![10, 20, 30, 40]);
        } else {
            panic!("Expected RawData for red channel");
        }

        if let Some(ChannelBytes::RawData(green)) = section.green {
            assert_eq!(green, vec![50, 60, 70, 80]);
        } else {
            panic!("Expected RawData for green channel");
        }

        if let Some(ChannelBytes::RawData(blue)) = section.blue {
            assert_eq!(blue, vec![90, 100, 110, 120]);
        } else {
            panic!("Expected RawData for blue channel");
        }
    }

    #[test]
    fn test_image_data_section_zip_with_prediction_16bit() {
        let width = 2;
        let height = 1;
        let channel_count = 1;

        // 16-bit data: [1000, 1100]
        // In Big Endian: [3, 232, 4, 76]
        // Applying prediction (stride 2):
        // [3, 232, (4 - 3), (76 - 232)] = [3, 232, 1, 100] (wrapping sub)
        let mut data = vec![3, 232, 1, 100];

        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut bytes = vec![0, 3]; // ZIP with prediction
        bytes.extend_from_slice(&compressed);

        let section = ImageDataSection::from_bytes(
            &bytes,
            PsdDepth::Sixteen,
            width,
            height,
            channel_count as u8,
        )
        .unwrap();

        assert_eq!(section.compression, PsdChannelCompression::ZipWithPrediction);

        // Decompressed: [3, 232, 4, 76]
        // Downsampled to 8-bit: [1000 / 256, 1100 / 256] = [3, 4]
        if let ChannelBytes::RawData(red) = section.red {
            assert_eq!(red, vec![3, 4]);
        } else {
            panic!("Expected RawData for red channel");
        }
    }
}
