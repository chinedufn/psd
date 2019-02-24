use crate::psd_channel::PsdChannelCompression;
use crate::sections::PsdCursor;
use failure::Error;

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
    pub(in crate) compression: PsdChannelCompression,
    /// The red channel of the final image
    pub(in crate) red: ChannelBytes,
    /// The green channel of the final image
    pub(in crate) green: ChannelBytes,
    /// the blue channel of the final image
    pub(in crate) blue: ChannelBytes,
    /// the alpha channel of the final image.
    /// If there is no alpha channel then it is a fully opaque image.
    pub(in crate) alpha: Option<ChannelBytes>,
}

impl ImageDataSection {
    /// Create an ImageDataSection from the bytes in the corresponding section in a PSD file
    /// (including the length market)
    pub fn from_bytes(
        bytes: &[u8],
        height: u32,
        channel_count: u8,
    ) -> Result<ImageDataSection, Error> {
        let mut cursor = PsdCursor::new(bytes);
        let channel_count = channel_count as usize;

        let compression = cursor.read_u16()?;
        let compression = PsdChannelCompression::new(compression)?;

        let (red, green, blue, alpha) = match compression {
            PsdChannelCompression::RawData => {
                // First 2 bytes were compression bytes
                let channel_bytes = &bytes[2..];
                let channel_byte_count = channel_bytes.len();

                let bytes_per_channel = channel_byte_count / channel_count;

                // First bytes are red
                let red = channel_bytes[..bytes_per_channel].into();
                // Next bytes are green
                let green = channel_bytes[bytes_per_channel..2 * bytes_per_channel].into();
                // Then comes blue
                let blue = channel_bytes[2 * bytes_per_channel..3 * bytes_per_channel].into();
                // And optionally alpha bytes
                let alpha = match channel_count {
                    4 => Some(ChannelBytes::RawData(
                        channel_bytes[3 * bytes_per_channel..4 * bytes_per_channel].to_vec(),
                    )),
                    3 => None,
                    // This was was handled above by returning an error
                    _ => unreachable!(),
                };

                (
                    ChannelBytes::RawData(red),
                    ChannelBytes::RawData(green),
                    ChannelBytes::RawData(blue),
                    alpha,
                )
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
                let mut green_byte_count = 0;
                let mut blue_byte_count = 0;
                let mut alpha_byte_count = if channel_count == 4 { Some(0) } else { None };

                for _ in 0..height {
                    red_byte_count += cursor.read_u16()? as usize;
                }
                for _ in 0..height {
                    green_byte_count += cursor.read_u16()? as usize;
                }
                for _ in 0..height {
                    blue_byte_count += cursor.read_u16()? as usize;
                }

                if let Some(ref mut alpha_byte_count) = alpha_byte_count {
                    for _ in 0..height {
                        *alpha_byte_count += cursor.read_u16()? as usize;
                    }
                }

                // 2 bytes for compression level, then 2 bytes for each scanline of each channel
                // We're skipping over the bytes that describe the length of each scanling since
                // we don't currently use them. We might re-think this in the future when we
                // implement serialization of a Psd back into bytes.. But not a concern at the
                // moment.
                let channel_data_start = 2 + (channel_count * height as usize * 2);

                let (red_start, red_end) =
                    (channel_data_start, channel_data_start + red_byte_count);

                let (green_start, green_end) = (red_end, red_end + green_byte_count);

                let (blue_start, blue_end) = (green_end, green_end + blue_byte_count);

                let red = bytes[red_start as usize..red_end as usize].into();
                let green = bytes[green_start as usize..green_end as usize].into();
                let blue = bytes[blue_start as usize..blue_end as usize].into();
                let alpha = match alpha_byte_count {
                    Some(alpha_byte_count) => {
                        let alpha_start = blue_end as usize;
                        let alpha_end = alpha_start + alpha_byte_count as usize;
                        Some(ChannelBytes::RleCompressed(
                            bytes[alpha_start..alpha_end].into(),
                        ))
                    }
                    None => None,
                };

                (
                    ChannelBytes::RleCompressed(red),
                    ChannelBytes::RleCompressed(green),
                    ChannelBytes::RleCompressed(blue),
                    alpha,
                )
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

#[derive(Debug)]
pub enum ChannelBytes {
    RawData(Vec<u8>),
    RleCompressed(Vec<u8>),
}
