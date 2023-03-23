use std::collections::HashMap;
use std::ops::Range;

use crate::psd_channel::PsdChannelCompression;
use crate::psd_channel::PsdChannelKind;
use crate::sections::image_data_section::ChannelBytes;
use crate::sections::layer_and_mask_information_section::groups::Groups;
use crate::sections::layer_and_mask_information_section::layer::{
    BlendMode, GroupDivider, LayerChannels, LayerRecord, PsdGroup, PsdLayer, PsdLayerError,
};
use crate::sections::layer_and_mask_information_section::layers::Layers;
use crate::sections::PsdCursor;

/// One of the possible additional layer block signatures
const SIGNATURE_EIGHT_BIM: [u8; 4] = [56, 66, 73, 77];
/// One of the possible additional layer block signatures
const SIGNATURE_EIGHT_B64: [u8; 4] = [56, 66, 54, 52];

/// Additional Layer Information constants.
/// Key of `Unicode layer name (Photoshop 5.0)`, "luni"
const KEY_UNICODE_LAYER_NAME: &[u8; 4] = b"luni";
/// Key of `Section divider setting (Photoshop 6.0)`, "lsct"
const KEY_SECTION_DIVIDER_SETTING: &[u8; 4] = b"lsct";

pub mod groups;
pub mod layer;
pub mod layers;

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
    pub(crate) layers: Layers,
    pub(crate) groups: Groups,
}

/// Frame represents a group stack frame
#[derive(Debug)]
struct Frame {
    start_idx: usize,
    name: String,
    group_id: u32,
    parent_group_id: u32,
}

impl LayerAndMaskInformationSection {
    /// Create a LayerAndMaskInformationSection from the bytes in the corresponding section in a
    /// PSD file (including the length marker).
    pub fn from_bytes(
        bytes: &[u8],
        psd_width: u32,
        psd_height: u32,
    ) -> Result<LayerAndMaskInformationSection, PsdLayerError> {
        let mut cursor = PsdCursor::new(bytes);

        // The first four bytes of the section is the length marker for the layer and mask
        // information section.
        //
        // We do not currently use it since the number of bytes passed into this function was
        // the exact number of bytes in the layer and information mask section of the PSD file,
        // so there's no way for us to accidentally read too many bytes. If we did the program
        // would panic.
        let len = cursor.read_u32();

        if len == 0 {
            return Ok(LayerAndMaskInformationSection {
                layers: Layers::new(),
                groups: Groups::with_capacity(0),
            });
        }

        // Read the next four bytes to get the length of the layer info section.
        let _layer_info_section_len = cursor.read_u32();

        // Next 2 bytes is the layer count
        //
        // NOTE: Appears to be -1 when we create a new PSD and don't create any new layers but
        // instead only manipulate the default background layer.
        //
        // # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
        //
        // Layer count. If it is a negative number, its absolute value is the number of layers and
        // the first alpha channel contains the transparency data for the merged result.
        let layer_count = cursor.read_i16();

        // TODO: If the layer count was negative we were supposed to treat the first alpha
        // channel as transparency data for the merged result.. So add a new test with a transparent
        // PSD and make sure that we're handling this case properly.
        let layer_count: u16 = layer_count.abs() as u16;
        let (group_count, layer_records) =
            LayerAndMaskInformationSection::read_layer_records(&mut cursor, layer_count)?;

        LayerAndMaskInformationSection::decode_layers(
            layer_records,
            group_count,
            (psd_width, psd_height),
        )
    }

    fn decode_layers(
        layer_records: Vec<(LayerRecord, LayerChannels)>,
        group_count: usize,
        psd_size: (u32, u32),
    ) -> Result<LayerAndMaskInformationSection, PsdLayerError> {
        let mut layers = Layers::with_capacity(layer_records.len());
        let mut groups = Groups::with_capacity(group_count);

        // Create stack with root-level
        let mut stack: Vec<Frame> = vec![Frame {
            start_idx: 0,
            name: String::from("root"),
            group_id: 0,
            parent_group_id: 0,
        }];

        // Viewed group counter
        let mut already_viewed = 0;

        // Read each layer's channel image data
        for (layer_record, channels) in layer_records.into_iter() {
            // get current group from stack
            let current_group_id = stack.last().unwrap().group_id;

            match layer_record.divider_type {
                // open the folder
                Some(GroupDivider::CloseFolder) | Some(GroupDivider::OpenFolder) => {
                    already_viewed = already_viewed + 1;

                    let frame = Frame {
                        start_idx: layers.len(),
                        name: layer_record.name,
                        group_id: already_viewed,
                        parent_group_id: current_group_id,
                    };

                    stack.push(frame);
                }

                // close the folder
                Some(GroupDivider::BoundingSection) => {
                    let frame = stack.pop().unwrap();

                    let range = Range {
                        start: frame.start_idx,
                        end: layers.len(),
                    };

                    groups.push(PsdGroup::new(
                        frame.name,
                        frame.group_id,
                        range,
                        &layer_record,
                        psd_size.0,
                        psd_size.1,
                        if frame.parent_group_id > 0 {
                            Some(frame.parent_group_id)
                        } else {
                            None
                        },
                    ));
                }

                _ => {
                    let psd_layer = LayerAndMaskInformationSection::read_layer(
                        &layer_record,
                        current_group_id,
                        psd_size,
                        channels,
                    )?;

                    layers.push(psd_layer.name.clone(), psd_layer);
                }
            };
        }

        Ok(LayerAndMaskInformationSection { layers, groups })
    }

    fn read_layer_records(
        cursor: &mut PsdCursor,
        layer_count: u16,
    ) -> Result<(usize, Vec<(LayerRecord, LayerChannels)>), PsdLayerError> {
        let mut groups_count = 0;

        let mut layer_records = vec![];
        // Read each layer record
        for _layer_num in 0..layer_count {
            let layer_record = read_layer_record(cursor)?;

            match layer_record.divider_type {
                Some(GroupDivider::BoundingSection) => {
                    groups_count = groups_count + 1;
                }
                _ => {}
            }

            layer_records.push(layer_record);
        }

        let mut result = vec![];
        for layer_record in layer_records {
            let channels = read_layer_channels(
                cursor,
                &layer_record.channel_data_lengths,
                layer_record.height() as usize,
            )?;

            result.push((layer_record, channels));
        }

        // Photoshop stores layers in reverse order
        result.reverse();
        Ok((groups_count, result))
    }

    fn read_layer(
        layer_record: &LayerRecord,
        parent_id: u32,
        psd_size: (u32, u32),
        channels: LayerChannels,
    ) -> Result<PsdLayer, PsdLayerError> {
        Ok(PsdLayer::new(
            &layer_record,
            psd_size.0,
            psd_size.1,
            if parent_id > 0 { Some(parent_id) } else { None },
            channels,
        ))
    }
}

/// Reads layer channels
fn read_layer_channels(
    cursor: &mut PsdCursor,
    channel_data_lengths: &Vec<(PsdChannelKind, u32)>,
    scanlines: usize,
) -> Result<LayerChannels, PsdLayerError> {
    let capacity = channel_data_lengths.len();
    let mut channels = HashMap::with_capacity(capacity);

    for (channel_kind, channel_length) in channel_data_lengths.iter() {
        let compression = cursor.read_u16();
        let compression = PsdChannelCompression::new(compression)
            .ok_or(PsdLayerError::InvalidCompression { compression })?;

        let compression = if *channel_length > 0 {
            compression
        } else {
            PsdChannelCompression::RawData
        };

        let channel_data = cursor.read(*channel_length);
        let channel_bytes = match compression {
            PsdChannelCompression::RawData => ChannelBytes::RawData(channel_data.into()),
            PsdChannelCompression::RleCompressed => {
                // We're skipping over the bytes that describe the length of each scanline since
                // we don't currently use them. We might re-think this in the future when we
                // implement serialization of a Psd back into bytes.. But not a concern at the
                // moment.
                // Compressed bytes per scanline are encoded at the beginning as 2 bytes
                // per scanline
                let channel_data = &channel_data[2 * scanlines..];
                ChannelBytes::RleCompressed(channel_data.into())
            }
            _ => unimplemented!("Zip compression currently unsupported"),
        };

        channels.insert(*channel_kind, channel_bytes);
    }

    Ok(channels)
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
fn read_layer_record(cursor: &mut PsdCursor) -> Result<LayerRecord, PsdLayerError> {
    let mut channel_data_lengths = vec![];

    // FIXME:
    // Ran into a bug where a PSD file had a top and left of over 4billion.
    // The PSD file was 128x128 yet the single layer in the file was 1024x1024.
    // Manually changing the layer's dimensions fixed the problem.. but this is something
    // that we should look into handling automatically since the file opened just fine in
    // Photoshop.

    // Read the rectangle that encloses the layer mask.
    let top = cursor.read_i32();

    let left = cursor.read_i32();

    // Subtract one in order to zero index. If a layer is fully transparent it's bottom will
    // already be 0 so we don't subtract
    let bottom = cursor.read_i32();
    let bottom = if bottom == 0 { 0 } else { bottom - 1 };

    // Subtract one in order to zero index. If a layer is fully transparent it's right will
    // already be zero so we don't subtract.
    let right = cursor.read_i32();
    let right = if right == 0 { 0 } else { right - 1 };

    // Get the number of channels in the layer
    let channel_count = cursor.read_u16();

    // Read the channel information
    for _ in 0..channel_count {
        let channel_id = cursor.read_i16();
        let channel_id =
            PsdChannelKind::new(channel_id).ok_or(PsdLayerError::InvalidChannel { channel_id })?;

        let channel_length = cursor.read_u32();
        // The first two bytes encode the compression, the rest of the bytes
        // are the channel data.
        let channel_data_length = channel_length - 2;

        channel_data_lengths.push((channel_id, channel_data_length));
    }

    // We do not currently parse the blend mode signature, skip it
    cursor.read_4();

    let mut key = [0; 4];
    key.copy_from_slice(cursor.read_4());
    let blend_mode = match BlendMode::match_mode(key) {
        Some(v) => v,
        None => return Err(PsdLayerError::UnknownBlendingMode { mode: key }),
    };

    let opacity = cursor.read_u8();

    let clipping_base = cursor.read_u8();
    let clipping_base = clipping_base == 0;

    // We do not currently parse all flags, only visible
    // Flags:
    //  - bit 0 = transparency protected;
    //  - bit 1 = visible;
    //  - bit 2 = obsolete;
    //  - bit 3 = 1 for Photoshop 5.0 and later, tells if bit 4 has useful information;
    //  - bit 4 = pixel data irrelevant to appearance of document
    let visible = cursor.read_u8() & (1 << 1) != 0; // here we get second bit - visible

    // We do not currently parse the filler, skip it
    cursor.read_1();

    // We do not currently use the length of the extra data field, skip it
    cursor.read_4();

    // We do not currently use the layer mask data, skip it
    let layer_mask_data_len = cursor.read_u32();
    cursor.read(layer_mask_data_len);

    // We do not currently use the layer blending range, skip it
    let layer_blending_range_data_len = cursor.read_u32();
    cursor.read(layer_blending_range_data_len);

    // Read the layer name
    let name_len = cursor.read_u8();
    let name = cursor.read(name_len as u32);
    let name = String::from_utf8_lossy(name);
    let mut name = name.to_string();

    // Layer name is padded to the next multiple of 4 bytes.
    // So if the name length is 9, there will be three throwaway bytes
    // after it. Here we skip over those throwaday bytes.
    //
    // The 1 is the 1 byte that we read for the name length
    let bytes_mod_4 = (name_len + 1) % 4;
    let padding = (4 - bytes_mod_4) % 4;
    cursor.read(padding as u32);

    let mut divider_type = None;
    // There can be multiple additional layer information sections so we'll loop
    // until we stop seeing them.
    while cursor.peek_4() == SIGNATURE_EIGHT_BIM || cursor.peek_4() == SIGNATURE_EIGHT_B64 {
        let _signature = cursor.read_4();
        let mut key = [0; 4];
        key.copy_from_slice(cursor.read_4());
        let additional_layer_info_len = cursor.read_u32();

        match &key {
            KEY_UNICODE_LAYER_NAME => {
                let pos = cursor.position();
                name = cursor.read_unicode_string_padding(1);
                cursor.seek(pos + additional_layer_info_len as u64);
            }
            KEY_SECTION_DIVIDER_SETTING => {
                divider_type = GroupDivider::match_divider(cursor.read_i32());

                // data present only if length >= 12
                if additional_layer_info_len >= 12 {
                    let _signature = cursor.read_4();
                    let _key = cursor.read_4();
                }

                // data present only if length >= 16
                if additional_layer_info_len >= 16 {
                    cursor.read_4();
                }
            }

            // TODO: Skipping other keys until we implement parsing for them
            _ => {
                cursor.read(additional_layer_info_len);
            }
        }
    }

    Ok(LayerRecord {
        name,
        channel_data_lengths,
        top,
        left,
        bottom,
        right,
        visible,
        opacity,
        clipping_base,
        blend_mode,
        divider_type,
    })
}
