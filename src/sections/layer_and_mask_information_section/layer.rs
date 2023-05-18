use std::collections::HashMap;
use std::ops::{Deref, Range};

use thiserror::Error;

use crate::psd_channel::IntoRgba;
use crate::psd_channel::PsdChannelCompression;
use crate::psd_channel::PsdChannelError;
use crate::psd_channel::PsdChannelKind;
use crate::sections::image_data_section::ChannelBytes;

/// Information about a layer in a PSD file.
///
/// TODO: I set all of these pub during a late evening of getting to get things working.
/// Replace with private and accessor methods so that this crate is as locked down as possible
/// (to allow us to be strict).
#[derive(Debug, Clone)]
pub struct LayerProperties {
    /// The name of this layer
    pub(crate) name: String,
    /// The position of the top of the layer
    pub(crate) layer_top: i32,
    /// The position of the left of the layer
    pub(crate) layer_left: i32,
    /// The position of the bottom of the layer
    pub(crate) layer_bottom: i32,
    /// The position of the right of the layer
    pub(crate) layer_right: i32,
    /// If true, the layer is marked as visible
    pub(crate) visible: bool,
    /// The opacity of the layer
    pub(crate) opacity: u8,
    /// If true, the layer is clipping mask
    pub(crate) clipping_mask: bool,
    /// The width of the PSD
    pub(crate) psd_width: u32,
    /// The height of the PSD
    pub(crate) psd_height: u32,
    /// Blending mode of the layer
    pub(crate) blend_mode: BlendMode,
    /// If layer is nested, contains parent group ID, otherwise `None`
    pub(crate) group_id: Option<u32>,
}

impl LayerProperties {
    pub fn from_layer_record(
        name: String,
        layer_record: &LayerRecord,
        psd_width: u32,
        psd_height: u32,
        group_id: Option<u32>,
    ) -> Self {
        LayerProperties {
            name,
            layer_top: layer_record.top,
            layer_left: layer_record.left,
            layer_bottom: layer_record.bottom,
            layer_right: layer_record.right,
            opacity: layer_record.opacity,
            clipping_mask: layer_record.clipping_base,
            visible: layer_record.visible,
            blend_mode: layer_record.blend_mode,
            psd_width,
            psd_height,
            group_id,
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

    /// If true, the layer is marked as visible
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// The position of the top of the layer
    pub fn layer_top(&self) -> i32 {
        self.layer_top
    }

    /// The position of the left of the layer
    pub fn layer_left(&self) -> i32 {
        self.layer_left
    }

    /// The position of the bottom of the layer
    pub fn layer_bottom(&self) -> i32 {
        self.layer_bottom
    }

    /// The position of the right of the layer
    pub fn layer_right(&self) -> i32 {
        self.layer_right
    }

    /// The opacity of the layer
    pub fn opacity(&self) -> u8 {
        self.opacity
    }

    /// If true, the layer is clipping mask
    pub fn is_clipping_mask(&self) -> bool {
        self.clipping_mask
    }

    /// Returns blending mode of the layer
    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    /// If layer is nested, returns parent group ID, otherwise `None`
    pub fn parent_id(&self) -> Option<u32> {
        self.group_id
    }
}

/// PsdGroup represents a group of layers
#[derive(Debug, Clone)]
pub struct PsdGroup {
    /// Group unique identifier
    pub(crate) id: u32,
    /// Idx range of contained layers
    pub(crate) contained_layers: Range<usize>,
    /// Common layer properties
    pub(crate) layer_properties: LayerProperties,
}

impl PsdGroup {
    /// Create a new photoshop group layer
    pub fn new(
        name: String,
        id: u32,
        contained_layers: Range<usize>,
        layer_record: &LayerRecord,
        psd_width: u32,
        psd_height: u32,
        group_id: Option<u32>,
    ) -> Self {
        let layer_properties =
            LayerProperties::from_layer_record(name, layer_record, psd_width, psd_height, group_id);

        PsdGroup {
            id,
            contained_layers,
            layer_properties,
        }
    }

    /// A unique identifier for the layer within the PSD file
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Deref for PsdGroup {
    type Target = LayerProperties;

    fn deref(&self) -> &Self::Target {
        &self.layer_properties
    }
}

/// Channels represents channels of the layer, stored separately.
pub type LayerChannels = HashMap<PsdChannelKind, ChannelBytes>;

/// PsdLayer represents a pixel layer
#[derive(Debug, Clone)]
pub struct PsdLayer {
    /// The channels of the layer, stored separately.
    ///
    /// You can combine these channels into a final image. For example, you might combine
    /// the Red, Green and Blue channels, or you might also combine the TransparencyMask (alpha)
    /// channel, or you might make use of the layer masks.
    ///
    /// Storing the channels separately allows for this flexability.
    pub(crate) channels: LayerChannels,
    /// Common layer properties
    pub(crate) layer_properties: LayerProperties,
}

/// An error when working with a PsdLayer
#[derive(Debug, PartialEq, Error)]
pub enum PsdLayerError {
    #[error(
        r#"Could not combine Red, Green, Blue and Alpha.
        This layer is missing channel: {channel:#?}"#
    )]
    MissingChannels { channel: PsdChannelKind },
    #[error("{channel_id} is an invalid channel id, must be 0, 1, 2, -1, -2, or -3.")]
    InvalidChannel { channel_id: i16 },
    #[error(r#"Unknown blending mode: {mode:#?}"#)]
    UnknownBlendingMode { mode: [u8; 4] },
    #[error("{compression} is an invalid layer channel compression. Must be 0, 1, 2 or 3")]
    InvalidCompression { compression: u16 },
}

impl PsdLayer {
    /// Create a new photoshop layer
    pub fn new(
        layer_record: &LayerRecord,
        psd_width: u32,
        psd_height: u32,
        group_id: Option<u32>,
        channels: LayerChannels,
    ) -> PsdLayer {
        PsdLayer {
            layer_properties: LayerProperties::from_layer_record(
                layer_record.name.clone(),
                layer_record,
                psd_width,
                psd_height,
                group_id,
            ),
            channels,
        }
    }

    /// Get the compression level for one of this layer's channels
    pub fn compression(
        &self,
        channel: PsdChannelKind,
    ) -> Result<PsdChannelCompression, PsdChannelError> {
        match self.channels.get(&channel) {
            Some(channel) => match channel {
                ChannelBytes::RawData(_) => Ok(PsdChannelCompression::RawData),
                ChannelBytes::RleCompressed(_) => Ok(PsdChannelCompression::RleCompressed),
            },
            None => Err(PsdChannelError::ChannelNotFound { channel }),
        }
    }

    /// Create a vector that interleaves the red, green, blue and alpha channels in this PSD
    ///
    /// vec![R, G, B, A, R, G, B, A, ...]
    pub fn rgba(&self) -> Vec<u8> {
        self.generate_rgba()
    }

    // Get one of the PsdLayerChannels of this PsdLayer
    fn get_channel(&self, channel: PsdChannelKind) -> Option<&ChannelBytes> {
        self.channels.get(&channel)
    }
}

impl Deref for PsdLayer {
    type Target = LayerProperties;

    fn deref(&self) -> &Self::Target {
        &self.layer_properties
    }
}

/// GroupDivider represents tag type of Section divider.
#[derive(Debug, Clone)]
pub(super) enum GroupDivider {
    /// 0 = any other type of layer
    Other = 0,
    /// 1 = open "folder"
    OpenFolder = 1,
    /// 2 = closed "folder"
    CloseFolder = 2,
    ///  3 = bounding section divider, hidden in the Photoshop UI
    BoundingSection = 3,
}

impl GroupDivider {
    pub(super) fn match_divider(divider: i32) -> Option<GroupDivider> {
        match divider {
            0 => Some(GroupDivider::Other),
            1 => Some(GroupDivider::OpenFolder),
            2 => Some(GroupDivider::CloseFolder),
            3 => Some(GroupDivider::BoundingSection),
            _ => None,
        }
    }
}

/// Describes how to blend a layer with the layer below it
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub enum BlendMode {
    PassThrough = 0,
    Normal = 1,
    Dissolve = 2,
    Darken = 3,
    Multiply = 4,
    ColorBurn = 5,
    LinearBurn = 6,
    DarkerColor = 7,
    Lighten = 8,
    Screen = 9,
    ColorDodge = 10,
    LinearDodge = 11,
    LighterColor = 12,
    Overlay = 13,
    SoftLight = 14,
    HardLight = 15,
    VividLight = 16,
    LinearLight = 17,
    PinLight = 18,
    HardMix = 19,
    Difference = 20,
    Exclusion = 21,
    Subtract = 22,
    Divide = 23,
    Hue = 24,
    Saturation = 25,
    Color = 26,
    Luminosity = 27,
}

impl BlendMode {
    pub(super) fn match_mode(mode: [u8; 4]) -> Option<BlendMode> {
        match &mode {
            b"pass" => Some(BlendMode::PassThrough),
            b"norm" => Some(BlendMode::Normal),
            b"diss" => Some(BlendMode::Dissolve),
            b"dark" => Some(BlendMode::Darken),
            b"mul " => Some(BlendMode::Multiply),
            b"idiv" => Some(BlendMode::ColorBurn),
            b"lbrn" => Some(BlendMode::LinearBurn),
            b"dkCl" => Some(BlendMode::DarkerColor),
            b"lite" => Some(BlendMode::Lighten),
            b"scrn" => Some(BlendMode::Screen),
            b"div " => Some(BlendMode::ColorDodge),
            b"lddg" => Some(BlendMode::LinearDodge),
            b"lgCl" => Some(BlendMode::LighterColor),
            b"over" => Some(BlendMode::Overlay),
            b"sLit" => Some(BlendMode::SoftLight),
            b"hLit" => Some(BlendMode::HardLight),
            b"vLit" => Some(BlendMode::VividLight),
            b"lLit" => Some(BlendMode::LinearLight),
            b"pLit" => Some(BlendMode::PinLight),
            b"hMix" => Some(BlendMode::HardMix),
            b"diff" => Some(BlendMode::Difference),
            b"smud" => Some(BlendMode::Exclusion),
            b"fsub" => Some(BlendMode::Subtract),
            b"fdiv" => Some(BlendMode::Divide),
            b"hue " => Some(BlendMode::Hue),
            b"sat " => Some(BlendMode::Saturation),
            b"colr" => Some(BlendMode::Color),
            b"lum " => Some(BlendMode::Luminosity),
            _ => None,
        }
    }
}

/// A layer record within the layer info section
///
/// TODO: Set all ofo these pubs to get things working. Replace with private
/// and accessor methods
#[derive(Debug, Clone)]
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
    pub(super) top: i32,
    /// The position of the left of the image
    pub(super) left: i32,
    /// The position of the bottom of the image
    pub(super) bottom: i32,
    /// The position of the right of the image
    pub(super) right: i32,
    /// If true, the layer is marked as visible
    pub(super) visible: bool,
    /// The opacity of the layer
    pub(super) opacity: u8,
    /// If true, the layer is clipping mask
    pub(super) clipping_base: bool,
    /// Blending mode of the layer
    pub(super) blend_mode: BlendMode,
    /// Group divider tag
    pub(super) divider_type: Option<GroupDivider>,
}

impl LayerRecord {
    /// The height of this layer record
    pub fn height(&self) -> i32 {
        (self.bottom - self.top) + 1
    }
}

impl IntoRgba for PsdLayer {
    /// A layer might take up only a subsection of a PSD, so if when iterating through
    /// the pixels in a layer we need to transform the pixel's location before placing
    /// it into the RGBA for the entire PSD.
    ///
    /// Given this illustration:
    ///
    /// ┌──────────────────────────────────────┐
    /// │                                      │
    /// │  Entire Psd                          │
    /// │                                      │
    /// │         ┌─────────────────────────┐  │
    /// │         │                         │  │
    /// │         │ Layer                   │  │
    /// │         │                         │  │
    /// │         │                         │  │
    /// │         └─────────────────────────┘  │
    /// │                                      │
    /// └──────────────────────────────────────┘
    ///
    /// The top left pixel in the layer will have index 0, but within the PSD
    /// that idx will be much more than 0 since there are some rows of pixels
    /// above it.
    ///
    /// So we transform the pixel's index based on the layer's left and top
    /// position within the PSD.
    fn rgba_idx(&self, idx: usize) -> Option<usize> {
        let left_in_layer = idx % self.width() as usize;
        let left_in_psd = self.layer_properties.layer_left + left_in_layer as i32;

        let top_in_layer = idx / self.width() as usize;
        let top_in_psd = self.layer_properties.layer_top + top_in_layer as i32;

        let idx = top_in_psd
            .checked_mul(self.layer_properties.psd_width as i32)
            .unwrap()
            + left_in_psd;

        if idx < 0 {
            None
        } else {
            Some(idx as usize)
        }
    }

    fn red(&self) -> &ChannelBytes {
        self.get_channel(PsdChannelKind::Red).unwrap()
    }

    fn green(&self) -> Option<&ChannelBytes> {
        self.get_channel(PsdChannelKind::Green)
    }

    fn blue(&self) -> Option<&ChannelBytes> {
        self.get_channel(PsdChannelKind::Blue)
    }

    fn alpha(&self) -> Option<&ChannelBytes> {
        self.get_channel(PsdChannelKind::TransparencyMask)
    }

    fn psd_width(&self) -> u32 {
        self.layer_properties.psd_width
    }

    fn psd_height(&self) -> u32 {
        self.layer_properties.psd_height
    }
}
