use crate::sections::image_data_section::ChannelBytes;
use crate::sections::PsdCursor;
use failure::{format_err, Error, Fail};
use std::collections::HashMap;

/// Information about a layer in a PSD file.
///
/// TODO: Set all ofo these pubs to get things working. Replace with private
/// and accessor methods
#[derive(Debug)]
pub struct PsdLayer {
    /// The name of this layer
    pub(super) name: String,
    /// The channels of the layer, stored separately.
    ///
    /// You can combine these channels into a final image. For example, you might combine
    /// the Red, Green and Blue channels, or you might also combine the TransparencyMask (alpha)
    /// channel, or you might make use of the layer masks.
    ///
    /// Storing the channels separately allows for this flexability.
    pub(super) channels: HashMap<PsdLayerChannelKind, ChannelBytes>,
    /// The position of the top of the image
    pub(crate) layer_top: u32,
    /// The position of the left of the image
    pub(crate) layer_left: u32,
    /// The position of the bottom of the image
    pub(crate) layer_bottom: u32,
    /// The position of the right of the image
    pub(crate) layer_right: u32,
    /// The width of the PSD
    pub(crate) psd_width: u32,
    /// The height of the PSD
    pub(crate) psd_height: u32,
}

/// An error when working with a PsdLayer
#[derive(Debug, Fail)]
pub enum PsdLayerError {
    #[fail(
        display = r#"Could not combine Red, Green, Blue and Alpha.
        This layer is missing channel: {:#?}"#,
        channel
    )]
    MissingChannels { channel: PsdLayerChannelKind },
}

impl PsdLayer {
    /// Create a new photoshop layer
    pub fn new(
        name: String,
        layer_top: u32,
        layer_left: u32,
        layer_bottom: u32,
        layer_right: u32,
        psd_width: u32,
        psd_height: u32,
    ) -> PsdLayer {
        PsdLayer {
            name,
            channels: HashMap::new(),
            layer_top,
            layer_left,
            layer_bottom,
            layer_right,
            psd_width,
            psd_height,
        }
    }

    /// Get the name of the layer
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The width of the layer
    pub fn width(&self) -> u16 {
        (self.layer_right - self.layer_left) as u16
    }

    /// The height of the layer
    pub fn height(&self) -> u16 {
        (self.layer_bottom - self.layer_top) as u16
    }

    /// Create a vector that interleaves the red, green, blue and alpha channels in this PSD
    ///
    /// vec![R, G, B, A, R, G, B, A, ...]
    pub fn rgba(&self) -> Result<Vec<u8>, Error> {
        let red = self.get_channel(PsdLayerChannelKind::Red)?;
        let green = self.get_channel(PsdLayerChannelKind::Green)?;
        let blue = self.get_channel(PsdLayerChannelKind::Blue)?;
        let alpha = self.get_channel(PsdLayerChannelKind::TransparencyMask)?;

        let mut rgba = vec![0; self.psd_width as usize * self.psd_height as usize * 4 as usize];

        self.insert_channel_bytes(&mut rgba, 0, &red);

        self.insert_channel_bytes(&mut rgba, 1, &green);

        self.insert_channel_bytes(&mut rgba, 2, &blue);

        self.insert_channel_bytes(&mut rgba, 3, &alpha);

        Ok(rgba)
    }

    // Get one of the PsdLayerChannels of this PsdLayer
    fn get_channel(&self, channel: PsdLayerChannelKind) -> Result<&ChannelBytes, Error> {
        match self.channels.get(&channel) {
            Some(layer_channel) => Ok(layer_channel),
            None => Err(PsdLayerError::MissingChannels { channel })?,
        }
    }

    // FIXME: Normalize with image data section insert channel bytes
    fn insert_channel_bytes(
        &self,
        rgba: &mut Vec<u8>,
        // FIXME: Replace offset with channel_kind and use .rgba_offset()
        offset: usize,
        channel_bytes: &ChannelBytes,
    ) {
        match channel_bytes {
            ChannelBytes::RawData(channel_bytes) => {
                let start = (self.layer_top * self.psd_width) + self.layer_left;
                let start = start as usize;

                for (idx, byte) in channel_bytes.iter().enumerate() {
                    rgba[(start + idx) * 4 + offset] = *byte;
                }
            }
            // https://en.wikipedia.org/wiki/PackBits
            ChannelBytes::RleCompressed(channel_bytes) => {
                self.rle_decompress_channel(rgba, offset, &channel_bytes);
            }
        }
    }

    // https://en.wikipedia.org/wiki/PackBits
    // TODO: Normalize with image_data_section.rs rle compression code
    fn rle_decompress_channel(&self, rgba: &mut Vec<u8>, offset: usize, channel_bytes: &Vec<u8>) {
        let mut cursor = PsdCursor::new(&channel_bytes[..]);

        let start = (self.layer_top * self.psd_width) + self.layer_left;
        let start = start as usize;

        let mut idx = 0;

        while cursor.position() != cursor.get_ref().len() as u64 {
            let header = cursor.read_i8().unwrap() as i16;

            if header == -128 {
                continue;
            } else if header >= 0 {
                let bytes_to_read = 1 + header;
                for byte in cursor.read(bytes_to_read as u32).unwrap() {
                    rgba[(start + idx) * 4 + offset] = *byte;
                    idx += 1;
                }
            } else {
                let repeat = 1 - header;
                let byte = cursor.read_1().unwrap()[0];
                for _ in 0..repeat as usize {
                    rgba[(start + idx) * 4 + offset] = byte;
                    idx += 1;
                }
            };
        }
    }
}

/// A layer record within the layer info section
///
/// TODO: Set all ofo these pubs to get things working. Replace with private
/// and accessor methods
#[derive(Debug)]
pub struct LayerRecord {
    /// The name of the layer
    pub(super) name: String,
    /// The channels that this record has and the number of bytes in each channel.
    ///
    /// Each channel has one byte per pixel in the PSD.
    ///
    /// So a 1x1 image would have 1 byte per channel.
    ///
    /// A 2x2 image would have 4 bytes per channel.
    pub(super) channel_data_lengths: Vec<(PsdLayerChannelKind, u32)>,
    /// The position of the top of the image
    pub(super) top: u32,
    /// The position of the left of the image
    pub(super) left: u32,
    /// The position of the bottom of the image
    pub(super) bottom: u32,
    /// The position of the right of the image
    pub(super) right: u32,
}

/// A channel within a PSD Layer
#[derive(Debug)]
pub struct PsdLayerChannel {
    /// How the channel data is compressed
    pub(super) compression: PsdLayerChannelCompression,
    /// The data for this image channel
    pub(super) channel_data: Vec<u8>,
}

/// How is this layer channel data compressed?
#[derive(Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum PsdLayerChannelCompression {
    /// Not compressed
    RawData = 0,
    /// Compressed using [PackBits RLE compression](https://en.wikipedia.org/wiki/PackBits)
    RleCompressed = 1,
    /// Currently unsupported
    ZipWithoutPrediction = 2,
    /// Currently unsupported
    ZipWithPrediction = 3,
}

impl PsdLayerChannelCompression {
    /// Create a new PsdLayerChannelCompression
    pub fn new(compression: u16) -> Result<PsdLayerChannelCompression, Error> {
        match compression {
            0 => Ok(PsdLayerChannelCompression::RawData),
            1 => Ok(PsdLayerChannelCompression::RleCompressed),
            2 => Ok(PsdLayerChannelCompression::ZipWithoutPrediction),
            3 => Ok(PsdLayerChannelCompression::ZipWithPrediction),
            _ => Err(PsdLayerChannelError::InvalidCompression { compression })?,
        }
    }
}

/// The different kinds of channels in a layer (red, green, blue, ...).
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[allow(missing_docs)]
pub enum PsdLayerChannelKind {
    Red = 0,
    Green = 1,
    Blue = 2,
    TransparencyMask = -1,
    UserSuppliedLayerMask = -2,
    RealUserSuppliedLayerMask = -3,
}

/// Represents an invalid layer channel id
#[derive(Debug, Fail)]
pub enum PsdLayerChannelError {
    #[fail(
        display = "{} is an invalid channel id, must be 0, 1, 2, -1, -2, or -3.",
        channel_id
    )]
    InvalidChannel { channel_id: i16 },
    #[fail(
        display = "{} is an invalid layer channel compression. Must be 0, 1, 2 or 3",
        compression
    )]
    InvalidCompression { compression: u16 },
}

impl PsdLayerChannelKind {
    /// Create a new PsdLayerChannel
    pub fn new(channel_id: i16) -> Result<PsdLayerChannelKind, Error> {
        match channel_id {
            0 => Ok(PsdLayerChannelKind::Red),
            1 => Ok(PsdLayerChannelKind::Green),
            2 => Ok(PsdLayerChannelKind::Blue),
            -1 => Ok(PsdLayerChannelKind::TransparencyMask),
            -2 => Ok(PsdLayerChannelKind::UserSuppliedLayerMask),
            -3 => Ok(PsdLayerChannelKind::RealUserSuppliedLayerMask),
            _ => Err(PsdLayerChannelError::InvalidChannel { channel_id })?,
        }
    }

    /// R -> 0
    /// G -> 1
    /// B -> 2
    /// A -> 3
    pub fn rgba_offset(&self) -> Result<usize, Error> {
        match self {
            PsdLayerChannelKind::Red => Ok(0),
            PsdLayerChannelKind::Green => Ok(1),
            PsdLayerChannelKind::Blue => Ok(2),
            PsdLayerChannelKind::TransparencyMask => Ok(3),
            _ => Err(format_err!("{:#?} is not an RGBA channel", &self)),
        }
    }
}
