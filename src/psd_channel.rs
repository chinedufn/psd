use crate::sections::image_data_section::ChannelBytes;
use crate::sections::{PsdCursor, PsdSerialize};
use thiserror::Error;

pub trait IntoRgba {
    /// Given an index of a pixel in the current rectangle
    /// (top left is 0.. to the right of that is 1.. etc) return the index of that pixel in the
    /// RGBA image that will be generated.
    ///
    /// If the final image or layer is the size of the PSD then this will return the same idx,
    /// otherwise it will get transformed.
    ///
    /// index could be `None` if layer's top or left is negative.
    ///
    /// index could be bigger than the size of the image if layer's bottom, right, width, height is bigger than image.
    fn rgba_idx(&self, idx: usize) -> Option<usize>;

    /// The first channel
    fn red(&self) -> &ChannelBytes;

    /// The second channel
    fn green(&self) -> Option<&ChannelBytes>;

    /// The third channel
    fn blue(&self) -> Option<&ChannelBytes>;

    /// The fourth channel
    fn alpha(&self) -> Option<&ChannelBytes>;

    /// The width of the PSD
    fn psd_width(&self) -> u32;

    /// The height of the PSD
    fn psd_height(&self) -> u32;

    fn generate_rgba(&self) -> Vec<u8> {
        let rgba_len = (self.psd_width() * self.psd_height() * 4) as usize;

        let red = self.red();
        let green = self.green();
        let blue = self.blue();
        let alpha = self.alpha();

        // TODO: We're assuming that if we only see two channels it is a 16 bit grayscale
        //  PSD. Instead we should just check the Psd's color mode and depth to see if
        //  they are grayscale and sixteen. As we run into more cases we'll clean things like
        //  this up over time.
        //        if green.is_some() && blue.is_none() && alpha.is_none() {
        //            return self.generate_16_bit_grayscale_rgba();
        //        }

        let mut rgba = vec![0; rgba_len];

        use crate::psd_channel::PsdChannelKind::*;

        self.insert_channel_bytes(&mut rgba, Red, red);

        // If there is a green channel we use it, otherwise we use the red channel since this is
        // a single channel grey image (such as a heightmap).
        if let Some(green) = green {
            self.insert_channel_bytes(&mut rgba, Green, green);
        } else {
            self.insert_channel_bytes(&mut rgba, Green, red);
        }

        // If there is a blue channel we use it, otherwise we use the red channel since this is
        // a single channel grey image (such as a heightmap).
        if let Some(blue) = blue {
            self.insert_channel_bytes(&mut rgba, Blue, blue);
        } else {
            self.insert_channel_bytes(&mut rgba, Blue, red);
        }

        if let Some(alpha_channel) = alpha {
            self.insert_channel_bytes(&mut rgba, TransparencyMask, alpha_channel);
        } else {
            // If there is no transparency data then the image is opaque
            for idx in 0..rgba_len / 4 {
                rgba[idx * 4 + 3] = 255;
            }
        }

        rgba
    }

    /// Generate an RGBA Vec<u8> from a composite image or layer that uses 16 bits per
    /// pixel. We do this by mapping the 16 bits back down to 8 bits.
    ///
    /// The 16 bits are stored across the red and green channels (first and second).
    fn generate_16_bit_grayscale_rgba(&self) -> Vec<u8> {
        match self.red() {
            ChannelBytes::RawData(red) => match self.green().unwrap() {
                ChannelBytes::RawData(green) => sixteen_to_eight_rgba(red, green),
                ChannelBytes::RleCompressed(green) => {
                    let green = &rle_decompress(green);
                    sixteen_to_eight_rgba(red, green)
                }
                ChannelBytes::RleCompressedScanlines { data, .. } => {
                    let green = &rle_decompress(data);
                    sixteen_to_eight_rgba(red, green)
                }
            },
            ChannelBytes::RleCompressed(red) => {
                let red = &rle_decompress(red);
                match self.green().unwrap() {
                    ChannelBytes::RawData(green) => sixteen_to_eight_rgba(red, green),
                    ChannelBytes::RleCompressed(green) => {
                        let green = &rle_decompress(green);
                        sixteen_to_eight_rgba(red, green)
                    }
                    ChannelBytes::RleCompressedScanlines { data, .. } => {
                        let green = &rle_decompress(data);
                        sixteen_to_eight_rgba(red, green)
                    }
                }
            }
            ChannelBytes::RleCompressedScanlines { data, .. } => {
                let red = &rle_decompress(data);
                match self.green().unwrap() {
                    ChannelBytes::RawData(green) => sixteen_to_eight_rgba(red, green),
                    ChannelBytes::RleCompressed(green) => {
                        let green = &rle_decompress(green);
                        sixteen_to_eight_rgba(red, green)
                    }
                    ChannelBytes::RleCompressedScanlines { data, .. } => {
                        let green = &rle_decompress(data);
                        sixteen_to_eight_rgba(red, green)
                    }
                }
            }
        }
    }

    /// Given some vector of bytes, insert the bytes from the given channel into the vector.
    ///
    /// Doing it this way allows us to allocate for one vector and insert all 4 (RGBA) channels into
    /// it.
    fn insert_channel_bytes(
        &self,
        rgba: &mut Vec<u8>,
        channel_kind: PsdChannelKind,
        channel_bytes: &ChannelBytes,
    ) {
        match channel_bytes {
            ChannelBytes::RawData(channel_bytes) => {
                let offset = channel_kind.rgba_offset().unwrap();

                for (idx, byte) in channel_bytes.iter().enumerate() {
                    if let Some(rgba_idx) = self.rgba_idx(idx) {
                        rgba[rgba_idx * 4 + offset] = *byte;
                    }
                }
            }
            // https://en.wikipedia.org/wiki/PackBits
            ChannelBytes::RleCompressed(channel_bytes) => {
                self.insert_rle_channel(rgba, channel_kind, &channel_bytes);
            }
            ChannelBytes::RleCompressedScanlines { data, .. } => {
                self.insert_rle_channel(rgba, channel_kind, &data);
            }
        }
    }

    /// rle decompress a channel (R,G,B or A) and insert it into a vector of RGBA pixels.
    ///
    /// We use the channels offset to know where to put it.. So red would go in 0, 4, 8..
    /// blue would go in 1, 5, 9.. etc
    ///
    /// https://en.wikipedia.org/wiki/PackBits - algorithm used for decompression
    fn insert_rle_channel(
        &self,
        rgba: &mut Vec<u8>,
        channel_kind: PsdChannelKind,
        channel_bytes: &[u8],
    ) {
        let mut cursor = PsdCursor::new(&channel_bytes[..]);

        let mut idx = 0;
        let offset = channel_kind.rgba_offset().unwrap();
        let len = cursor.get_ref().len() as u64;

        while cursor.position() < len {
            let header = cursor.read_i8() as i16;

            if header == -128 {
                continue;
            } else if header >= 0 {
                let bytes_to_read = 1 + header;
                if cursor.position() + bytes_to_read as u64 > len {
                    break;
                }
                for byte in cursor.read(bytes_to_read as u32) {
                    if let Some(rgba_idx) = self.rgba_idx(idx) {
                        if let Some(buffer) = rgba.get_mut(rgba_idx * 4 + offset) {
                            *buffer = *byte;
                        }
                    }

                    idx += 1;
                }
            } else {
                let repeat = 1 - header;

                if cursor.position() + 1 > len {
                    break;
                }
                let byte = cursor.read_1()[0];
                for _ in 0..repeat {
                    if let Some(rgba_idx) = self.rgba_idx(idx) {
                        if let Some(buffer) = rgba.get_mut(rgba_idx * 4 + offset) {
                            *buffer = byte;
                        }
                    }

                    idx += 1;
                }
            };
        }
    }
}

/// Rle decompress a channel
#[allow(dead_code)]
pub(crate) fn rle_decompress(bytes: &[u8]) -> Vec<u8> {
    let mut cursor = PsdCursor::new(&bytes[..]);

    let mut decompressed = vec![];

    while cursor.position() != cursor.get_ref().len() as u64 {
        let header = cursor.read_i8() as i16;

        if header == -128 {
            continue;
        } else if header >= 0 {
            let bytes_to_read = 1 + header;
            for byte in cursor.read(bytes_to_read as u32) {
                decompressed.push(*byte);
            }
        } else {
            let repeat = 1 - header;
            let byte = cursor.read_1()[0];
            for _ in 0..repeat {
                decompressed.push(byte);
            }
        };
    }

    decompressed
}

/// RLE (PackBits) compress a planar channel buffer by scanlines.
/// Returns (per-scanline byte lengths, concatenated compressed data).
pub(crate) fn rle_compress_scanlines(
    input: &[u8],
    width: usize,
    height: usize,
    bytes_per_sample: usize,
) -> (Vec<u16>, Vec<u8>) {
    assert_eq!(input.len(), width * height * bytes_per_sample);
    let mut lengths = Vec::with_capacity(height);
    let mut data = Vec::new();

    for row in 0..height {
        let start = row * width * bytes_per_sample;
        let end = start + width * bytes_per_sample;
        let line = &input[start..end];
        let compressed = packbits_compress(line);
        lengths.push(compressed.len() as u16);
        data.extend_from_slice(&compressed);
    }

    (lengths, data)
}

fn packbits_compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        // Check for run
        let mut run_len = 1;
        while i + run_len < input.len() && input[i] == input[i + run_len] && run_len < 128 {
            run_len += 1;
        }
        if run_len >= 3 {
            // Emit run
            out.push((1i16 - run_len as i16) as i8 as u8); // -(run_len-1) - 1 => 1 - run_len
            out.push(input[i]);
            i += run_len;
        } else {
            // Emit literal up to next run or 128 bytes
            let lit_start = i;
            let mut lit_len = 0;
            while i < input.len() {
                // Look ahead for a run of >=3
                if i + 2 < input.len() && input[i] == input[i + 1] && input[i] == input[i + 2] {
                    break;
                }
                i += 1;
                lit_len += 1;
                if lit_len == 128 { break; }
            }
            out.push((lit_len as u8) - 1);
            out.extend_from_slice(&input[lit_start..lit_start + lit_len]);
        }
    }
    out
}

/// Take two 8 bit channels that together represent a 16 bit channel and convert them down
/// into an 8 bit channel.
///
/// We store the final bytes in the first channel (overwriting the old bytes)
#[allow(dead_code)]
fn sixteen_to_eight_rgba(channel1: &[u8], channel2: &[u8]) -> Vec<u8> {
    let mut eight = Vec::with_capacity(channel1.len());

    for c in [channel1, channel2].iter() {
        for idx in 0..c.len() {
            if idx % 2 == 1 {
                continue;
            }

            let sixteen_bit = [c[idx], c[idx + 1]];
            let sixteen_bit = u16::from_be_bytes(sixteen_bit);

            let eight_bit = (sixteen_bit / 256) as u8;

            eight.push(eight_bit);
            eight.push(eight_bit);
            eight.push(eight_bit);
            eight.push(255);
        }
    }

    eight
}

/// Indicates how a channels data is compressed
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
    pub fn new(compression: u16) -> Option<PsdChannelCompression> {
        match compression {
            0 => Some(PsdChannelCompression::RawData),
            1 => Some(PsdChannelCompression::RleCompressed),
            2 => Some(PsdChannelCompression::ZipWithoutPrediction),
            3 => Some(PsdChannelCompression::ZipWithPrediction),
            _ => None,
        }
    }
}

impl PsdSerialize for PsdChannelCompression {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        buffer.write((self.to_owned() as u16).to_be_bytes());
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

impl PsdSerialize for PsdChannelKind {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        buffer.write((self.to_owned() as i16).to_be_bytes());
    }
}

/// Represents an invalid channel
#[derive(Debug, Error)]
pub enum PsdChannelError {
    #[error("Channel {channel:#?} not present")]
    ChannelNotFound { channel: PsdChannelKind },
}

impl PsdChannelKind {
    /// Create a new PsdLayerChannel
    pub fn new(channel_id: i16) -> Option<PsdChannelKind> {
        match channel_id {
            0 => Some(PsdChannelKind::Red),
            1 => Some(PsdChannelKind::Green),
            2 => Some(PsdChannelKind::Blue),
            -1 => Some(PsdChannelKind::TransparencyMask),
            -2 => Some(PsdChannelKind::UserSuppliedLayerMask),
            -3 => Some(PsdChannelKind::RealUserSuppliedLayerMask),
            _ => None,
        }
    }

    /// R -> 0
    /// G -> 1
    /// B -> 2
    /// A -> 3
    pub fn rgba_offset(self) -> Result<usize, String> {
        match self {
            PsdChannelKind::Red => Ok(0),
            PsdChannelKind::Green => Ok(1),
            PsdChannelKind::Blue => Ok(2),
            PsdChannelKind::TransparencyMask => Ok(3),
            _ => Err(format!("{:#?} is not an RGBA channel", &self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sections::layer_and_mask_information_section::layer::{
        BlendMode, LayerChannels, LayerProperties,
    };
    use crate::PsdLayer;

    use super::*;

    /// Verify that when inserting an RLE channel's bytes into an RGBA byte vec we do not attempt to
    /// read beyond the channel's length.
    #[test]
    fn does_not_read_beyond_rle_channels_bytes() {
        let layer_properties = LayerProperties {
            name: "".into(),
            layer_top: 0,
            layer_left: 0,
            layer_bottom: 0,
            layer_right: 0,
            visible: true,
            opacity: 0,
            clipping_mask: false,
            psd_width: 1,
            psd_height: 1,
            blend_mode: BlendMode::Normal,
            group_id: None,
        };

        let layer = PsdLayer {
            channels: LayerChannels::from([(
                PsdChannelKind::Red,
                ChannelBytes::RleCompressed(vec![0, 0, 0]),
            )]),
            layer_properties,
        };

        let mut rgba = vec![0; (layer.width() * layer.height() * 4) as usize];

        layer.insert_channel_bytes(&mut rgba, PsdChannelKind::Red, layer.red());

        assert_eq!(rgba, vec![0; 4]);
    }
}
