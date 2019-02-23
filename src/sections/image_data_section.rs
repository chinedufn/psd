use crate::sections::layer_and_mask_information_section::layer::PsdLayerChannelCompression;
use crate::sections::PsdCursor;
use failure::{Error, Fail};

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
    pub(in crate) compression: PsdLayerChannelCompression,
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
    pub fn from_bytes(bytes: &[u8], width: u32, height: u32) -> Result<ImageDataSection, Error> {
        let mut cursor = PsdCursor::new(bytes);

        let compression = cursor.read_u16()?;
        let compression = PsdLayerChannelCompression::new(compression)?;

        let (red, green, blue, alpha) = match compression {
            PsdLayerChannelCompression::RawData => {
                // First 2 bytes were compression bytes
                let channel_bytes = &bytes[2..];
                let channel_byte_count = channel_bytes.len();
                let pixel_count = width * height;

                // Done this way instead of doing
                //   channel_count = channel_bytes.len() / pixel_count
                // so that we don't end up rounding to the nearest integer when in actuality
                // we had a few extra bytes that we weren't expecting.
                let channel_count = if channel_bytes.len() as u32 == pixel_count * 3 {
                    3
                } else if channel_bytes.len() as u32 == pixel_count * 4 {
                    4
                } else {
                    return Err(ImageDataSectionError::InvalidChannelCount {
                        channel_byte_count: channel_bytes.len(),
                    })?;
                };

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
            //
            // TODO: Normalize with layer.rs rle compression code
            PsdLayerChannelCompression::RleCompressed => {
                // The final image data has 3 channels, red, green, blue
                let channel_count = 3;

                let mut red_byte_count = 0;
                let mut green_byte_count = 0;
                let mut blue_byte_count = 0;

                for _ in 0..height {
                    red_byte_count += cursor.read_u16()? as u32;
                }
                for _ in 0..height {
                    green_byte_count += cursor.read_u16()? as u32;
                }
                for _ in 0..height {
                    blue_byte_count += cursor.read_u16()? as u32;
                }

                // 2 bytes for compression level, then 2 bytes for each scanline of each channel
                // We're skipping over the bytes that describe the length of each scanling since
                // we don't currently use them. We might re-think this in the future when we
                // implement serialization of a Psd back into bytes.. But not a concern at the
                // moment.
                let channel_data_start = 2 + (channel_count * height * 2);

                let (red_start, red_end) =
                    (channel_data_start, channel_data_start + red_byte_count);

                let (green_start, green_end) = (red_end, red_end + green_byte_count);

                let (blue_start, blue_end) = (green_end, green_end + blue_byte_count);

                let red = bytes[red_start as usize..red_end as usize].into();
                let green = bytes[green_start as usize..green_end as usize].into();
                let blue = bytes[blue_start as usize..blue_end as usize].into();

                (
                    ChannelBytes::RleCompressed(red),
                    ChannelBytes::RleCompressed(green),
                    ChannelBytes::RleCompressed(blue),
                    // FIXME: Add a test psd to transparency.rs that uses RLE compression
                    // but has transparency. Then this line will cause that to fail since
                    // we're expecting some transparency bytes.. So make it pass.
                    // Just need an 8x8 image with the first pixel blue, rest transparent
                    None,
                )
            }
            PsdLayerChannelCompression::ZipWithoutPrediction => unimplemented!(
                r#"Zip without prediction compression is currently unsupported.
                Please open an issue"#
            ),
            PsdLayerChannelCompression::ZipWithPrediction => unimplemented!(
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
    fn raw_data_rgb(cursor: &mut PsdCursor) -> Vec<u8> {
        unimplemented!();
    }

    // https://en.wikipedia.org/wiki/PackBits
    fn rle_rgb(cursor: &mut PsdCursor) -> Vec<u8> {
        unimplemented!();
    }
}

/// Represents an error when parsing the image data section
#[derive(Fail, Debug)]
pub enum ImageDataSectionError {
    /// After reading the first 2 bytes that specify the compression level for the image data
    /// section, if there is no compression and this is raw data then we should have either
    /// 3, or 4 channels per pixel. 3 if it is rgb, 4 if it is rgba.
    #[fail(
        display = "Found {} channel bytes, which is not a multiple of 3 or 4",
        channel_byte_count
    )]
    InvalidChannelCount { channel_byte_count: usize },
}

#[derive(Debug)]
pub enum ChannelBytes {
    RawData(Vec<u8>),
    RleCompressed(Vec<u8>),
}
