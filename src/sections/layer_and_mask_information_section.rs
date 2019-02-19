use crate::sections::PsdCursor;
use failure::{Error, Fail};
use std::collections::HashMap;
use std::io::Read;

/// One of the possible additional layer block signatures
const SIGNATURE_EIGHT_BIM: [u8; 4] = [56, 66, 73, 77];
/// One of the possible additional layer block signatures
const SIGNATURE_EIGHT_B64: [u8; 4] = [56, 66, 54, 52];

/// The LayerAndMaskInformationSection comes from the bytes in the fourth section of the PSD.
///
/// When possible we'll make the data easier to work with by storing it structures such as HashMaps.
///
/// # Note
///
/// We do not currently store all of the information that is present in the layer and mask
/// information section of the PSD. If something that you need is missing please open an issue.
///
/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// The fourth section of a Photoshop file contains information about layers and masks. This section of the document describes the formats of layer and mask records.
///
/// The complete merged image data is not stored here. The complete merged/composite image resides in the last section of the file. See See Image Data Section. If maximize compatibility is unchecked then the merged/composite is not created and the layer data must be read to reproduce the final image.
///
/// See Layer and mask information section shows the overall structure of this section. If there are no layers or masks, this section is just 4 bytes: the length field, which is set to zero. (**PSB** length is 8 bytes
///
/// 'Layr', 'Lr16' and 'Lr32' start at See Layer info. NOTE: The length of the section may already be known.)
///
/// When parsing this section pay close attention to the length of sections.
///
/// | Length   | Description                                                                                                                                                                                |
/// |----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | 4        | Length of the layer and mask information section.<br> (**PSB** length is 8 bytes.)                                                                                                         |
/// | Variable | Layer info (see See Layer info for details).                                                                                                                                               |
/// | Variable | Global layer mask info (see See Global layer mask info for details).                                                                                                                       |
/// | Variable | (Photoshop 4.0 and later) <br> Series of tagged blocks containing various types of data. See See Additional Layer Information for the list of the types of data that can be included here. |
#[derive(Debug)]
pub struct LayerAndMaskInformationSection {
    pub(in crate) layers: HashMap<String, PsdLayer>,
}

impl LayerAndMaskInformationSection {
    /// Create a LayerAndMaskInformationSection from the bytes in the corresponding section in a
    /// PSD file (including the length marker).
    pub fn from_bytes(bytes: &[u8]) -> Result<LayerAndMaskInformationSection, Error> {
        let mut cursor = PsdCursor::new(bytes);

        let mut layers = HashMap::new();

        // The first four bytes of the section is the length marker for the layer and mask
        // information section, we won't be needing it.
        cursor.read_4()?;

        // Read the next four bytes to get the length of the layer info section
        let layer_info_section_len = cursor.read_u32()?;

        // Next 2 bytes is the layer count
        let layer_count = cursor.read_u16()?;

        let mut layer_records = vec![];

        // Read each layer record
        for layer_num in 0..layer_count {
            layer_records.push(read_layer_record(&mut cursor)?);
        }

        // Read each layer's channel image data
        for layer_record in layer_records {
            let mut psd_layer = PsdLayer::new();

            for (channel_kind, channel_length) in layer_record.channel_data_lengths {
                let compression = cursor.read_u16()?;
                let compression = PsdLayerChannelCompression::new(compression)?;

                let channel_data = cursor.read(channel_length)?;

                psd_layer.channels.insert(
                    channel_kind,
                    PsdLayerChannel {
                        compression,
                        channel_data: channel_data.into(),
                    },
                );
            }

            layers.insert(layer_record.name, psd_layer);
        }

        Ok(LayerAndMaskInformationSection { layers })
    }
}

/// Read bytes, starting from the cursor, until we've processed all of the data for a layer in
/// the layer records section.
///
/// At the moment we skip over some of the data.
///
/// Please open an issue if there is data in here that you need that we don't currently parse.
///
/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// Information about each layer.
///
/// | Length                 | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
/// |------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | 4 * 4                  | Rectangle containing the contents of the layer. Specified as top, left, bottom, right coordinates                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
/// | 2                      | Number of channels in the layer                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
/// | 6 * number of channels | Channel information. Six bytes per channel,<br> consisting of: 2 bytes for Channel ID: 0 = red, 1 = green, etc.; <br> -1 = transparency mask; -2 = user supplied layer mask, -3 real user supplied layer mask (when both a user mask and a vector mask are present) <br>  4 bytes for length of corresponding channel data. (**PSB** 8 bytes for length of corresponding channel data.) See See Channel image data for structure of channel data.                                                                                                                                                 |
/// | 4                      | Blend mode signature: '8BIM'                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
/// | 4                      | Blend mode key: <br> 'pass' = pass through, 'norm' = normal, 'diss' = dissolve, 'dark' = darken, 'mul ' = multiply, 'idiv' = color burn, 'lbrn' = linear burn, 'dkCl' = darker color, 'lite' = lighten, 'scrn' = screen, 'div ' = color dodge, 'lddg' = linear dodge, 'lgCl' = lighter color, 'over' = overlay, 'sLit' = soft light, 'hLit' = hard light, 'vLit' = vivid light, 'lLit' = linear light, 'pLit' = pin light, 'hMix' = hard mix, 'diff' = difference, 'smud' = exclusion, 'fsub' = subtract, 'fdiv' = divide 'hue ' = hue, 'sat ' = saturation, 'colr' = color, 'lum ' = luminosity, |
/// | 1                      | Opacity. 0 = transparent ... 255 = opaque                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
/// | 1                      | Clipping: 0 = base, 1 = non-base                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
/// | 1                      | Flags: <br> bit 0 = transparency protected; <br> bit 1 = visible; <br> bit 2 = obsolete; <br> bit 3 = 1 for Photoshop 5.0 and later, tells if bit 4 has useful information; <br> bit 4 = pixel data irrelevant to appearance of document                                                                                                                                                                                                                                                                                                                                                          |
/// | 1                      | Filler (zero)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
/// | 4                      | Length of the extra data field ( = the total length of the next five fields).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
/// | Variable               | Layer mask data: See See Layer mask / adjustment layer data for structure. Can be 40 bytes, 24 bytes, or 4 bytes if no layer mask.                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
/// | Variable               | Layer blending ranges: See See Layer blending ranges data.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
/// | Variable               | Layer name: Pascal string, padded to a multiple of 4 bytes.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
fn read_layer_record(cursor: &mut PsdCursor) -> Result<LayerRecord, Error> {
    let mut channel_data_lengths = vec![];

    // We do not currently parse the layer rectangle, skip it
    let rectangle_bytes = 16;
    cursor.read(rectangle_bytes)?;

    // Get the number of channels in the layer
    let channel_count = cursor.read_u16()?;

    // Read the channel information
    for _ in 0..channel_count {
        let channel_id = cursor.read_i16()?;
        let channel_id = PsdLayerChannelKind::new(channel_id)?;

        let channel_length = cursor.read_u32()?;
        // The first two bytes encode the compression, the rest of the bytes
        // are the channel data.
        let channel_data_length = channel_length - 2;

        channel_data_lengths.push((channel_id, channel_data_length));
    }

    // We do not currently parse the blend mode signature, skip it
    cursor.read_4()?;

    // We do not currently parse the blend mode key, skip it
    cursor.read_4()?;

    // We do not currently parse the opacity, skip it
    cursor.read_1()?;

    // We do not currently parse the clipping, skip it
    cursor.read_1()?;

    // We do not currently parse the flags, skip it
    cursor.read_1()?;

    // We do not currently parse the filter, skip it
    cursor.read_1()?;

    // We do not currently use the length of the extra data field, skip it
    cursor.read_4()?;

    // We do not currently use the layer mask data, skip it
    let layer_mask_data_len = cursor.read_u32()?;
    cursor.read(layer_mask_data_len)?;

    // We do not currently use the layer blending range, skip it
    let layer_blending_range_data_len = cursor.read_u32()?;
    cursor.read(layer_blending_range_data_len)?;

    // Read the layer name
    let name_len = cursor.read_u8()?;
    let name = cursor.read(name_len as u32)?;
    let name = String::from_utf8_lossy(name);
    let name = name.to_string();

    // Layer name is padded to the next multiple of 4 bytes.
    // So if the name length is 9, there will be three throwaway bytes
    // after it. Here we skip over those throwaday bytes.
    //
    // The 1 is the 1 byte that we read for the name length
    let bytes_mod_4 = (name_len + 1) % 4;
    let padding = (4 - bytes_mod_4) % 4;
    cursor.read(padding as u32)?;

    // We do not currently handle additional layer information, so we skip it.
    //
    // There can be multiple additional layer information sections so we'll loop
    // until we stop seeing them.
    while cursor.peek_4()? == &SIGNATURE_EIGHT_BIM || cursor.peek_4()? == &SIGNATURE_EIGHT_B64 {
        let _signature = cursor.read_4()?;
        let _key = cursor.read_4()?;
        let additional_layer_info_len = cursor.read_u32()?;
        cursor.read(additional_layer_info_len)?;
    }

    Ok(LayerRecord {
        name,
        channel_data_lengths,
    })
}

/// Information about a layer in a PSD file.
#[derive(Debug)]
pub struct PsdLayer {
    /// The channels of the layer, stored separately.
    ///
    /// You can combine these channels into a final image. For example, you might combine
    /// the Red, Green and Blue channels, or you might also combine the TransparencyMask (alpha)
    /// channel, or you might make use of the layer masks.
    ///
    /// Storing the channels separately allows for this flexability.
    channels: HashMap<PsdLayerChannelKind, PsdLayerChannel>,
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
    pub fn new() -> PsdLayer {
        PsdLayer {
            channels: HashMap::new(),
        }
    }

    /// Create a vector that interleaves the red, green, blue and alpha channels in this PSD
    ///
    /// vec![R, G, B, A, R, G, B, A, ...]
    pub fn rgba(&self) -> Result<Vec<u8>, Error> {
        let red = self.get_channel(PsdLayerChannelKind::Red)?;
        let green = self.get_channel(PsdLayerChannelKind::Green)?;
        let blue = self.get_channel(PsdLayerChannelKind::Blue)?;
        let alpha = self.get_channel(PsdLayerChannelKind::TransparencyMask)?;

        let mut rgba = vec![];

        for idx in 0..red.channel_data.len() {
            rgba.push(red.channel_data[idx]);
            rgba.push(green.channel_data[idx]);
            rgba.push(blue.channel_data[idx]);
            rgba.push(alpha.channel_data[idx]);
        }

        Ok(rgba)
    }

    // Get one of the PsdLayerChannels of this PsdLayer
    fn get_channel(&self, channel: PsdLayerChannelKind) -> Result<&PsdLayerChannel, Error> {
        match self.channels.get(&channel) {
            Some(layer_channel) => Ok(layer_channel),
            None => Err(PsdLayerError::MissingChannels { channel })?,
        }
    }
}

/// A layer record within the layer info section
#[derive(Debug)]
struct LayerRecord {
    /// The name of the layer
    name: String,
    /// The channels that this record has and the number of bytes in each channel.
    ///
    /// Each channel has one byte per pixel in the PSD.
    ///
    /// So a 1x1 image would have 1 byte per channel.
    ///
    /// A 2x2 image would have 4 bytes per channel.
    channel_data_lengths: Vec<(PsdLayerChannelKind, u32)>,
}

/// A channel within a PSD Layer
#[derive(Debug)]
pub struct PsdLayerChannel {
    /// How the channel data is compressed
    compression: PsdLayerChannelCompression,
    /// The data for this image channel
    channel_data: Vec<u8>,
}

/// How is this layer channel data compressed?
#[derive(Debug)]
#[allow(missing_docs)]
pub enum PsdLayerChannelCompression {
    RawData = 0,
    RleCompressed = 1,
    ZipWithoutPrediction = 2,
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
}
