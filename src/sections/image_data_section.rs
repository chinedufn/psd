use crate::psd_channel::PsdChannelCompression;
use crate::sections::PsdCursor;
use crate::PsdDepth;
use thiserror::Error;

use super::PsdSerialize;

/// Represents a malformed image data
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

impl PsdSerialize for ImageDataSection {
    fn write<T>(&self, buffer: &mut super::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        use crate::sections::image_data_section::ChannelBytes::*;
        self.compression.write(buffer);

        match self.compression {
            PsdChannelCompression::RawData => {
                self.red.write(buffer);
                if let Some(green) = &self.green { green.write(buffer); }
                if let Some(blue) = &self.blue { blue.write(buffer); }
                if let Some(alpha) = &self.alpha { alpha.write(buffer); }
            }
            PsdChannelCompression::RleCompressed => {
                // For merged image RLE, Photoshop expects all scanline byte counts for all channels first,
                // then the concatenated compressed data for each channel.
                let mut channels: Vec<&ChannelBytes> = Vec::new();
                channels.push(&self.red);
                if let Some(g) = &self.green { channels.push(g); }
                if let Some(b) = &self.blue { channels.push(b); }
                if let Some(a) = &self.alpha { channels.push(a); }

                // Write scanline lengths for all channels
                for ch in channels.iter() {
                    match ch {
                        RleCompressedScanlines { scanline_lengths, .. } => {
                            for &len in scanline_lengths.iter() {
                                buffer.write((len as u16).to_be_bytes());
                            }
                        }
                        // Fallback: if we don't have lengths, write nothing (best-effort). Builders should supply lengths.
                        _ => {}
                    }
                }
                // Write data blobs for all channels
                for ch in channels.iter() {
                    match ch {
                        RleCompressedScanlines { data, .. } => buffer.write(data),
                        RleCompressed(bytes) => buffer.write(bytes),
                        RawData(bytes) => buffer.write(bytes),
                    }
                }
            }
            _ => {
                // Keep existing behavior for unsupported compressions
                self.red.write(buffer);
                if let Some(green) = &self.green { green.write(buffer); }
                if let Some(blue) = &self.blue { blue.write(buffer); }
                if let Some(alpha) = &self.alpha { alpha.write(buffer); }
            }
        }
    }
}
impl ImageDataSection {
    /// Create an ImageDataSection from the bytes in the corresponding section in a PSD file
    /// (including the length market)
    pub fn from_bytes(
        bytes: &[u8],
        depth: PsdDepth,
        psd_height: u32,
        channel_count: u8,
    ) -> Result<ImageDataSection, ImageDataSectionError> {
        let mut cursor = PsdCursor::new(bytes);
        let channel_count = channel_count as usize;

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
                let mut red = channel_bytes[..bytes_per_channel].into();

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
                // Read per-scanline byte counts for each channel
                let mut red_lengths: Vec<u16> = Vec::with_capacity(psd_height as usize);
                for _ in 0..psd_height { red_lengths.push(cursor.read_u16()); }

                let mut green_lengths: Option<Vec<u16>> = if channel_count >= 2 { Some(Vec::with_capacity(psd_height as usize)) } else { None };
                if let Some(ref mut gl) = green_lengths { for _ in 0..psd_height { gl.push(cursor.read_u16()); } }

                let mut blue_lengths: Option<Vec<u16>> = if channel_count >= 3 { Some(Vec::with_capacity(psd_height as usize)) } else { None };
                if let Some(ref mut bl) = blue_lengths { for _ in 0..psd_height { bl.push(cursor.read_u16()); } }

                let mut alpha_lengths: Option<Vec<u16>> = if channel_count == 4 { Some(Vec::with_capacity(psd_height as usize)) } else { None };
                if let Some(ref mut al) = alpha_lengths { for _ in 0..psd_height { al.push(cursor.read_u16()); } }

                // After reading all lengths, the remaining bytes are channel data in order
                let channel_data_start = cursor.position() as usize;

                // Helper to sum lengths
                let sum_len = |lens: &Vec<u16>| -> usize { lens.iter().map(|&v| v as usize).sum() };

                let red_byte_count = sum_len(&red_lengths);
                let (red_start, red_end) = (channel_data_start, channel_data_start + red_byte_count);
                let red_data = bytes[red_start..red_end].to_vec();

                let (green, green_end) = match green_lengths {
                    Some(ref gl) => {
                        let green_start = red_end;
                        let green_end = green_start + sum_len(gl);
                        (Some(ChannelBytes::RleCompressedScanlines { scanline_lengths: gl.clone(), data: bytes[green_start..green_end].to_vec() }), green_end)
                    }
                    None => (None, red_end),
                };

                let (blue, blue_end) = match blue_lengths {
                    Some(ref bl) => {
                        let blue_start = green_end;
                        let blue_end = blue_start + sum_len(bl);
                        (Some(ChannelBytes::RleCompressedScanlines { scanline_lengths: bl.clone(), data: bytes[blue_start..blue_end].to_vec() }), blue_end)
                    }
                    None => (None, green_end),
                };

                let alpha = match alpha_lengths {
                    Some(ref al) => {
                        let alpha_start = blue_end;
                        let alpha_end = alpha_start + sum_len(al);
                        Some(ChannelBytes::RleCompressedScanlines { scanline_lengths: al.clone(), data: bytes[alpha_start..alpha_end].to_vec() })
                    }
                    None => None,
                };

                (ChannelBytes::RleCompressedScanlines { scanline_lengths: red_lengths, data: red_data }, green, blue, alpha)
            }
            PsdChannelCompression::ZipWithoutPrediction => unimplemented!(
                r#"Zip without prediction compression is currently unsupported.
                Please open an issue"#
            ),
            PsdChannelCompression::ZipWithPrediction => unimplemented!(
                r#"Zip with prediction compression is currently unsupported.
                Please open an issue"#
            ),
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
    /// RLE compressed bytes without scanline headers (used by reader for convenience)
    RleCompressed(Vec<u8>),
    /// RLE compressed with per-scanline byte lengths and concatenated data (used for writing)
    RleCompressedScanlines {
        /// Big-endian 2-byte lengths per scanline
        scanline_lengths: Vec<u16>,
        /// Concatenated compressed data of all scanlines
        data: Vec<u8>,
    },
}

impl PsdSerialize for ChannelBytes {
    fn write<T>(&self, buffer: &mut super::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        match self {
            Self::RawData(bytes) => buffer.write(bytes),
            Self::RleCompressed(bytes) => buffer.write(bytes),
            Self::RleCompressedScanlines { scanline_lengths, data } => {
                // Caller is responsible for writing compression header; this writes headers+data
                for &len in scanline_lengths.iter() {
                    buffer.write((len as u16).to_be_bytes());
                }
                buffer.write(data);
            }
        }
    }
}
