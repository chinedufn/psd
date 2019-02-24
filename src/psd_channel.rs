use crate::sections::image_data_section::ChannelBytes;
use crate::sections::PsdCursor;
use failure::{Fail, Error, format_err};

pub trait InsertChannelBytes {
    /// Given an index of a pixel in the current rectangle
    /// (top left is 0.. to the right of that is 1.. etc) return the index of that pixel in the
    /// RGBA image that will be generated.
    ///
    /// If the final image or layer is the size of the PSD then this will return the same idx,
    /// otherwise it will get transformed.
    fn rgba_idx(&self, idx: usize) -> usize;

    /// Given some vector of bytes, insert the bytes from the given channel into the vector.
    ///
    /// Doing it this way allows us to allocate for one vector and insert all 4 (RGBA) channels into
    /// it.
    fn insert_channel_bytes(
        &self,
        rgba: &mut Vec<u8>,
        channel_kind: &PsdChannelKind,
        channel_bytes: &ChannelBytes,
    ) {
        match channel_bytes {
            ChannelBytes::RawData(channel_bytes) => {
                let offset = channel_kind.rgba_offset().unwrap();

                for (idx, byte) in channel_bytes.iter().enumerate() {
                    let rgba_idx = self.rgba_idx(idx);
                    rgba[rgba_idx * 4 + offset] = *byte;
                }
            }
            // https://en.wikipedia.org/wiki/PackBits
            ChannelBytes::RleCompressed(channel_bytes) => {
                self.rle_decompress_channel(rgba, &channel_kind, &channel_bytes);
            }
        }
    }

    /// rle decompress a channel (R,G,B or A) and insert it into a vector of RGBA pixels.
    ///
    /// We use the channels offset to know where to put it.. So red would go in 0, 4, 8..
    /// blue would go in 1, 5, 9.. etc
    ///
    /// https://en.wikipedia.org/wiki/PackBits - algorithm used for decompression
    fn rle_decompress_channel(
        &self,
        rgba: &mut Vec<u8>,
        channel_kind: &PsdChannelKind,
        channel_bytes: &Vec<u8>,
    ) {
        let mut cursor = PsdCursor::new(&channel_bytes[..]);

        let mut idx = 0;
        let offset = channel_kind.rgba_offset().unwrap();

        while cursor.position() != cursor.get_ref().len() as u64 {
            let header = cursor.read_i8().unwrap() as i16;

            if header == -128 {
                continue;
            } else if header >= 0 {
                let bytes_to_read = 1 + header;
                for byte in cursor.read(bytes_to_read as u32).unwrap() {
                    let rgba_idx = self.rgba_idx(idx);
                    rgba[rgba_idx * 4 + offset] = *byte;

                    idx += 1;
                }
            } else {
                let repeat = 1 - header;
                let byte = cursor.read_1().unwrap()[0];
                for _ in 0..repeat as usize {
                    let rgba_idx = self.rgba_idx(idx);
                    rgba[rgba_idx * 4 + offset] = byte;

                    idx += 1;
                }
            };
        }
    }
}

/// How is this layer channel data compressed?
#[derive(Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum PsdChannelCompression {
    /// Not compressed
    RawData = 0,
    /// Compressed using [PackBits RLE compression](https://en.wikipedia.org/wiki/PackBits)
    RleCompressed = 1,
    /// Currently unsupported
    ZipWithoutPrediction = 2,
    /// Currently unsupported
    ZipWithPrediction = 3,
}

impl PsdChannelCompression {
    /// Create a new PsdLayerChannelCompression
    pub fn new(compression: u16) -> Result<PsdChannelCompression, Error> {
        match compression {
            0 => Ok(PsdChannelCompression::RawData),
            1 => Ok(PsdChannelCompression::RleCompressed),
            2 => Ok(PsdChannelCompression::ZipWithoutPrediction),
            3 => Ok(PsdChannelCompression::ZipWithPrediction),
            _ => Err(PsdChannelError::InvalidCompression { compression })?,
        }
    }
}

/// The different kinds of channels in a layer (red, green, blue, ...).
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[allow(missing_docs)]
pub enum PsdChannelKind {
    Red = 0,
    Green = 1,
    Blue = 2,
    TransparencyMask = -1,
    UserSuppliedLayerMask = -2,
    RealUserSuppliedLayerMask = -3,
}

/// Represents an invalid channel
#[derive(Debug, Fail)]
pub enum PsdChannelError {
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
    #[fail(display = "Channel {:#?} not present", channel)]
    ChannelNotFound { channel: PsdChannelKind },
}

impl PsdChannelKind {
    /// Create a new PsdLayerChannel
    pub fn new(channel_id: i16) -> Result<PsdChannelKind, Error> {
        match channel_id {
            0 => Ok(PsdChannelKind::Red),
            1 => Ok(PsdChannelKind::Green),
            2 => Ok(PsdChannelKind::Blue),
            -1 => Ok(PsdChannelKind::TransparencyMask),
            -2 => Ok(PsdChannelKind::UserSuppliedLayerMask),
            -3 => Ok(PsdChannelKind::RealUserSuppliedLayerMask),
            _ => Err(PsdChannelError::InvalidChannel { channel_id })?,
        }
    }

    /// R -> 0
    /// G -> 1
    /// B -> 2
    /// A -> 3
    pub fn rgba_offset(&self) -> Result<usize, Error> {
        match self {
            PsdChannelKind::Red => Ok(0),
            PsdChannelKind::Green => Ok(1),
            PsdChannelKind::Blue => Ok(2),
            PsdChannelKind::TransparencyMask => Ok(3),
            _ => Err(format_err!("{:#?} is not an RGBA channel", &self)),
        }
    }
}

