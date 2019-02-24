use crate::psd_channel::PsdChannelCompression;
use crate::psd_channel::PsdChannelKind;
use crate::sections::image_data_section::ChannelBytes;
use crate::sections::layer_and_mask_information_section::layer::LayerRecord;
use crate::sections::layer_and_mask_information_section::layer::PsdLayer;
use crate::sections::PsdCursor;
use failure::Error;
use std::collections::HashMap;

/// One of the possible additional layer block signatures
const SIGNATURE_EIGHT_BIM: [u8; 4] = [56, 66, 73, 77];
/// One of the possible additional layer block signatures
const SIGNATURE_EIGHT_B64: [u8; 4] = [56, 66, 54, 52];

pub mod layer;

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
    pub(in crate) layers: Vec<PsdLayer>,
    /// A map of layer name to index within our layers vector
    pub(in crate) layer_names: HashMap<String, usize>,
}

impl LayerAndMaskInformationSection {
    /// Create a LayerAndMaskInformationSection from the bytes in the corresponding section in a
    /// PSD file (including the length marker).
    pub fn from_bytes(
        bytes: &[u8],
        psd_width: u32,
        psd_height: u32,
    ) -> Result<LayerAndMaskInformationSection, Error> {
        let mut cursor = PsdCursor::new(bytes);

        let mut layers = vec![];
        let mut layer_names = HashMap::new();

        // The first four bytes of the section is the length marker for the layer and mask
        // information section, we won't be needing it.
        cursor.read_4()?;

        // Read the next four bytes to get the length of the layer info section
        let _layer_info_section_len = cursor.read_u32()?;

        // Next 2 bytes is the layer count
        //
        // NOTE: Appears to be -1 when we create a new PSD and don't create any new layers but
        // instead only manipulate the default background layer.
        //
        // # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
        //
        // Layer count. If it is a negative number, its absolute value is the number of layers and
        // the first alpha channel contains the transparency data for the merged result.
        let layer_count = cursor.read_i16()?;

        // TODO: If the layer count was negative we were supposed to treat the first alpha
        // channel as transparency data for the merged result.. So add a new test with a transparent
        // PSD and make sure that we're handling this case properly.
        let layer_count: u16 = layer_count.abs() as u16;

        let mut layer_records = vec![];

        // Read each layer record
        for _layer_num in 0..layer_count {
            layer_records.push(read_layer_record(&mut cursor)?);
        }

        // Read each layer's channel image data
        for (idx, layer_record) in layer_records.into_iter().enumerate() {
            let mut psd_layer = PsdLayer::new(
                layer_record.name.clone(),
                layer_record.top,
                layer_record.left,
                layer_record.bottom,
                layer_record.right,
                psd_width,
                psd_height,
            );

            let scanlines = layer_record.height() as usize;

            for (channel_kind, channel_length) in layer_record.channel_data_lengths {
                let compression = cursor.read_u16()?;
                let compression = PsdChannelCompression::new(compression)?;

                let channel_data = cursor.read(channel_length)?;

                let channel_bytes = match compression {
                    PsdChannelCompression::RawData => ChannelBytes::RawData(channel_data.into()),
                    PsdChannelCompression::RleCompressed => {
                        // We're skipping over the bytes that describe the length of each scanline since
                        // we don't currently use them. We might re-think this in the future when we
                        // implement serialization of a Psd back into bytes.. But not a concern at the
                        // moment.
                        // Compressed bytes per scanling are encoded at the beginning as 2 bytes
                        // per scanline
                        let channel_data = &channel_data[2 * scanlines..];

                        ChannelBytes::RleCompressed(channel_data.into())
                    }
                    _ => unimplemented!("Zip compression currently unsupported"),
                };

                psd_layer.channels.insert(channel_kind, channel_bytes);
            }

            layer_names.insert(layer_record.name, idx);
            layers.push(psd_layer);
        }

        Ok(LayerAndMaskInformationSection {
            layers,
            layer_names,
        })
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

    // Read the rectangle that encloses the layer mask.
    let top = cursor.read_u32()?;
    let left = cursor.read_u32()?;
    // Subtract one in order to zero index
    let bottom = cursor.read_u32()? - 1;
    // Subtract one in order to zero index
    let right = cursor.read_u32()? - 1;

    // Get the number of channels in the layer
    let channel_count = cursor.read_u16()?;

    // Read the channel information
    for _ in 0..channel_count {
        let channel_id = cursor.read_i16()?;
        let channel_id = PsdChannelKind::new(channel_id)?;

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
        top,
        left,
        bottom,
        right,
    })
}
