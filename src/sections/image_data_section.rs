use crate::sections::layer_and_mask_information_section::PsdLayerChannelCompression;
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
    pub(in crate) compression: PsdLayerChannelCompression,
    /// The RGB pixels for this image
    pub(in crate) rgb: Vec<u8>,
}

impl ImageDataSection {
    /// Create an ImageDataSection from the bytes in the corresponding section in a PSD file
    /// (including the length market)
    pub fn from_bytes(bytes: &[u8]) -> Result<ImageDataSection, Error> {
        let mut cursor = PsdCursor::new(bytes);

        let compression = cursor.read_u16()?;
        let compression = PsdLayerChannelCompression::new(compression)?;

        // 3 channels, RGB. First 2 bytes are compression bytes, the rest are RGB bytes.
        let bytes_per_channel = (bytes.len() - 2) / 3;
        let rgb_bytes = &bytes[2..];

        let mut rgb = vec![];

        for idx in 0..bytes_per_channel {
            rgb.push(rgb_bytes[idx * 3]);
            rgb.push(rgb_bytes[idx * 3 + 1]);
            rgb.push(rgb_bytes[idx * 3 + 2]);
        }

        Ok(ImageDataSection { compression, rgb })
    }
}
