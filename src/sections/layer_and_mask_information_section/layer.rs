use crate::psd_channel::InsertChannelBytes;
use crate::sections::image_data_section::ChannelBytes;
use crate::sections::PsdCursor;
use failure::{format_err, Error, Fail};
use std::collections::HashMap;
use crate::psd_channel::PsdChannelKind;
use crate::psd_channel::PsdChannelError;
use crate::psd_channel::PsdChannelCompression;

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
    pub(super) channels: HashMap<PsdChannelKind, ChannelBytes>,
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
    MissingChannels { channel: PsdChannelKind },
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
        // If left is at 0 and right is at 4, the width is 5
        (self.layer_right - self.layer_left) as u16 + 1
    }

    /// The height of the layer
    pub fn height(&self) -> u16 {
        // If top is at 0 and bottom is at 3, the height is 4
        (self.layer_bottom - self.layer_top) as u16 + 1
    }

    /// Get the compression level for one of this layer's channels
    pub fn compression(&self, channel: &PsdChannelKind) -> Result<PsdChannelCompression, Error> {
        match self.channels.get(channel) {
            Some(channel) => match channel {
                ChannelBytes::RawData(_) => Ok(PsdChannelCompression::RawData),
                ChannelBytes::RleCompressed(_) => Ok(PsdChannelCompression::RleCompressed),
                _ => unimplemented!(),
            },
            None => Err(PsdChannelError::ChannelNotFound { channel: *channel })?,
        }
    }

    /// Create a vector that interleaves the red, green, blue and alpha channels in this PSD
    ///
    /// vec![R, G, B, A, R, G, B, A, ...]
    pub fn rgba(&self) -> Result<Vec<u8>, Error> {
        let red = self.get_channel(PsdChannelKind::Red)?;
        let green = self.get_channel(PsdChannelKind::Green)?;
        let blue = self.get_channel(PsdChannelKind::Blue)?;
        let alpha = self.get_channel(PsdChannelKind::TransparencyMask)?;

        // FIXME: Default to 0, not 255. Just testing something.
        let mut rgba = vec![0; self.psd_width as usize * self.psd_height as usize * 4 as usize];

        self.insert_channel_bytes(&mut rgba, &PsdChannelKind::Red, &red);

        self.insert_channel_bytes(&mut rgba, &PsdChannelKind::Green, &green);

        self.insert_channel_bytes(&mut rgba, &PsdChannelKind::Blue, &blue);

        self.insert_channel_bytes(&mut rgba, &PsdChannelKind::TransparencyMask, &alpha);

        Ok(rgba)
    }

    // Get one of the PsdLayerChannels of this PsdLayer
    fn get_channel(&self, channel: PsdChannelKind) -> Result<&ChannelBytes, Error> {
        match self.channels.get(&channel) {
            Some(layer_channel) => Ok(layer_channel),
            None => Err(PsdLayerError::MissingChannels { channel })?,
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
    pub(super) channel_data_lengths: Vec<(PsdChannelKind, u32)>,
    /// The position of the top of the image
    pub(super) top: u32,
    /// The position of the left of the image
    pub(super) left: u32,
    /// The position of the bottom of the image
    pub(super) bottom: u32,
    /// The position of the right of the image
    pub(super) right: u32,
}

impl LayerRecord {
    /// The height of this layer record
    pub fn height(&self) -> u32 {
        (self.bottom - self.top) + 1
    }
}


impl InsertChannelBytes for PsdLayer {
    fn rgba_idx(&self, idx: usize) -> usize {
        let left_in_layer = idx % self.width() as usize;
        let left_in_psd = self.layer_left as usize + left_in_layer;

        let top_in_psd = idx / self.width() as usize + self.layer_top as usize;

        let rgba_idx = (top_in_psd * self.psd_width as usize) + left_in_psd;

        rgba_idx
    }
}
