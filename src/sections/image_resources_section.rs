use std::collections::HashMap;
use std::iter::Map;
use std::ops::Range;

use failure::{Error, Fail};

use crate::sections::image_resources_section::UnitFloatStructure::Angle;
use crate::sections::PsdCursor;

const EXPECTED_RESOURCE_BLOCK_SIGNATURE: [u8; 4] = [56, 66, 73, 77];
const RESOURCE_SLICES_INFO: i16 = 1050;

struct ImageResourcesBlock {
    resource_id: i16,
    name: String,
    data_range: Range<usize>,
}

#[derive(Debug)]
pub struct ImageResourcesSection {}

/// Represents an malformed resource block
#[derive(Debug, PartialEq, Fail)]
pub enum ImageResourcesSectionError {
    #[fail(
    display = r#"The first four bytes (indices 0-3) must always equal [56, 66, 73, 77],
         which in string form is '8BIM'."#
    )]
    InvalidSignature {},
}


impl ImageResourcesSection {
    pub fn from_bytes(bytes: &[u8]) -> Result<ImageResourcesSection, Error> {
        let mut cursor = PsdCursor::new(bytes);

        let length = cursor.read_u32()? as u64;
        let mut read = 0;
        while read < length {
            let pair = ImageResourcesSection::read_resource_block(&mut cursor)?;
            read = pair.0;

            let block = pair.1;
            match block.resource_id {
                RESOURCE_SLICES_INFO => ImageResourcesSection::read_slice_block(&cursor.get_ref()[block.data_range])?,
                _ => {}
            }
        }

        Ok(ImageResourcesSection {})
    }

    fn read_resource_block(cursor: &mut PsdCursor) -> Result<(u64, ImageResourcesBlock), Error> {
        // First four bytes must be '8BIM'
        let signature = cursor.read_4()?;
        if signature != EXPECTED_RESOURCE_BLOCK_SIGNATURE {
            return Err(ImageResourcesSectionError::InvalidSignature {}.into());
        }

        let resource_id = cursor.read_i16()?;
        let name = cursor.read_pascal_string()?;

        let data_len = cursor.read_u32()?;
        let pos = cursor.position() as usize;
        // Note: data length is padded to even.
        let data_len = data_len + data_len % 2;
        let data_range = Range {
            start: pos,
            end: pos + data_len as usize,
        };
        cursor.read(data_len)?;

        Ok((
            cursor.position(),
            ImageResourcesBlock {
                resource_id,
                name,
                data_range,
            },
        ))
    }

    fn read_slice_block(bytes: &[u8]) -> Result<(), Error> {
        let mut cursor = PsdCursor::new(bytes);

        let version = cursor.read_i32()?;
        if version != 6 {
            unimplemented!("Adobe Photoshop 6.0+ slice currently unsupported");
        }

        // We do not currently parse top of all the slices, skip it
        cursor.read_i32()?;
        // We do not currently parse left of all the slices, skip it
        cursor.read_i32()?;
        // We do not currently parse bottom of all the slices, skip it
        cursor.read_i32()?;
        // We do not currently parse right of all the slices, skip it
        cursor.read_i32()?;
        // We do not currently parse name of group of slices, skip it
        cursor.read_unicode_string()?;

        let number_of_slices = cursor.read_u32()?;
        for n in 0..number_of_slices {
            ImageResourcesSection::read_slice_body(&mut cursor)?;
            println!("section read: {}\n", cursor.position());
        }

        Ok(())
    }

    fn read_slice_body(cursor: &mut PsdCursor) -> Result<Option<DescriptorStructure>, Error> {
        let slice_id = cursor.read_i32()?;
        let group_id = cursor.read_i32()?;
        let origin = cursor.read_i32()?;
        println!("slice_id: {}, group_id: {}, origin: {}", slice_id, group_id, origin);
        // if origin = 1, Associated Layer ID is present
        if origin == 1 {
            cursor.read_i32()?;
        }

        // We do not currently parse name of group of slices, skip it
        cursor.read_unicode_string_padding(1)?;
        // We do not currently parse type, skip it
        cursor.read_i32()?;
        // We do not currently parse top, skip it
        let top = cursor.read_i32()?;
        // We do not currently parse left, skip it
        let left = cursor.read_i32()?;
        // We do not currently parse bottom, skip it
        let bottom = cursor.read_i32()?;
        // We do not currently parse right, skip it
        let right = cursor.read_i32()?;
        println!("top: {}, left: {}, bottom: {}, right: {}", top, left, bottom, right);
        // We do not currently parse URL, skip it
        cursor.read_unicode_string_padding(1)?;
        // We do not currently parse target, skip it
        cursor.read_unicode_string_padding(1)?;
        // We do not currently parse message skip it
        cursor.read_unicode_string_padding(1)?;
        // We do not currently parse alt tag skip it
        cursor.read_unicode_string_padding(1)?;
        // We do not currently parse cell text HTML flag, skip it
        cursor.read_1()?;
        // We do not currently parse cell text, skip it
        cursor.read_unicode_string_padding(1)?;
        // We do not currently parse horizontal alignment, skip it
        cursor.read_i32()?;
        // We do not currently parse vertical alignment, skip it
        cursor.read_i32()?;
        // We do not currently parse color, skip it
        // Note: in docs color is ARGB tuple
        cursor.read_i32()?;

        let pos = cursor.position();
        let descriptor_version = cursor.peek_4()?;
        println!("{:?}", descriptor_version);

        let r = if descriptor_version == [0, 0, 0, 16] {
            cursor.read_4()?;

            let descriptor = DescriptorStructure::read_descriptor_structure(cursor)?;
            if descriptor.class_id.as_slice() == [0, 0, 0, 0] {
                cursor.seek(pos);
            }

            Some(descriptor)
        } else {
            None
        };

        Ok(r)
    }
}

pub struct DescriptorStructure {
    pub name: String,
    pub fields: HashMap<String, DescriptorField>,
    pub class_id: Vec<u8>,
}

pub enum DescriptorField {
    Descriptor(DescriptorStructure),
    Reference(Vec<DescriptorField>),
    UnitFloat(UnitFloatStructure),
    Double(f64),
    Class(ClassStructure),
    String(String),
    EnumeratedReference(EnumeratedReference),
    Offset(OffsetStructure),
    Boolean(bool),
    Alias(AliasStructure),
    List(Vec<DescriptorField>),
    LargeInteger(i64),
    Integer(i32),
    EnumeratedDescriptor(EnumeratedDescriptor),
    RawData(Vec<u8>),

    /// Only Reference fields
    ///
    ///
    Property(PropertyStructure),
    Identifier(i32),
    Index(i32),
    Name(NameStructure),
}

pub struct PropertyStructure {
    pub name: String,
    pub class_id: Vec<u8>,
    pub key_id: Vec<u8>,
}

pub enum UnitFloatStructure {
    Angle(f64),
    Density(f64),
    Distance(f64),
    None,
    Percent(f64),
    Pixels(f64),
}

/// Unit float structure units keys
/// '#Ang' = angle: base degrees
const UNIT_FLOAT_ANGLE: &[u8; 4] = b"#Ang";
/// '#Rsl' = density: base per inch
const UNIT_FLOAT_DENSITY: &[u8; 4] = b"#Rsl";
/// '#Rlt' = distance: base 72ppi
const UNIT_FLOAT_DISTANCE: &[u8; 4] = b"#Rlt";
/// '#Nne' = none: coerced.
const UNIT_FLOAT_NONE: &[u8; 4] = b"#Nne";
/// '#Prc'= percent: unit value
const UNIT_FLOAT_PERCENT: &[u8; 4] = b"#Prc";
/// '#Pxl' = pixels: tagged unit value
const UNIT_FLOAT_PIXELS: &[u8; 4] = b"#Pxl";

pub struct ClassStructure {
    pub name: String,
    pub class_id: Vec<u8>,
}

pub struct EnumeratedReference {
    pub name: String,
    pub class_id: Vec<u8>,
    pub key_id: Vec<u8>,
    pub enum_field: Vec<u8>,
}

pub struct OffsetStructure {
    pub name: String,
    pub class_id: Vec<u8>,
    pub offset: u32,
}

pub struct AliasStructure {
    pub data: Vec<u8>,
}

pub struct EnumeratedDescriptor {
    pub type_field: Vec<u8>,
    pub enum_field: Vec<u8>,
}

pub struct NameStructure {
    pub name: String,
    pub class_id: Vec<u8>,
    pub value: String,
}

/// Descriptor structure OSType keys
/// 'obj ' = Reference
const OS_TYPE_REFERENCE: &[u8; 4] = b"obj ";
/// 'Objc' = Descriptor
const OS_TYPE_DESCRIPTOR: &[u8; 4] = b"Objc";
/// 'VlLs' = List
const OS_TYPE_LIST: &[u8; 4] = b"VlLs";
/// 'doub' = Double
const OS_TYPE_DOUBLE: &[u8; 4] = b"doub";
/// 'UntF' = Unit float
const OS_TYPE_UNIT_FLOAT: &[u8; 4] = b"UntF";
/// 'TEXT' = String
const OS_TYPE_TEXT: &[u8; 4] = b"TEXT";
/// 'enum' = Enumerated
const OS_TYPE_ENUMERATED: &[u8; 4] = b"enum";
/// 'long' = Integer
const OS_TYPE_INTEGER: &[u8; 4] = b"long";
/// 'comp' = Large Integer
const OS_TYPE_LARGE_INTEGER: &[u8; 4] = b"comp";
/// 'bool' = Boolean
const OS_TYPE_BOOL: &[u8; 4] = b"bool";
/// 'GlbO' = GlobalObject same as Descriptor
const OS_TYPE_GLOBAL_OBJECT: &[u8; 4] = b"GlbO";
/// 'type' = Class
const OS_TYPE_CLASS: &[u8; 4] = b"type";
/// 'GlbC' = Class
const OS_TYPE_CLASS2: &[u8; 4] = b"GlbC";
/// 'alis' = Alias
const OS_TYPE_ALIAS: &[u8; 4] = b"alis";
/// 'tdta' = Raw Data
const OS_TYPE_RAW_DATA: &[u8; 4] = b"tdta";


/// Reference structure OSType keys
/// 'prop' = Property
const OS_TYPE_PROPERTY: &[u8; 4] = b"prop";
/// 'Clss' = Class
const OS_TYPE_CLASS3: &[u8; 4] = b"Clss";
/// 'Clss' = Class
const OS_TYPE_ENUMERATED_REFERENCE: &[u8; 4] = b"Enmr";
/// 'rele' = Offset
const OS_TYPE_OFFSET: &[u8; 4] = b"rele";
/// 'Idnt' = Identifier
const OS_TYPE_IDENTIFIER: &[u8; 4] = b"Idnt";
/// 'indx' = Index
const OS_TYPE_INDEX: &[u8; 4] = b"indx";
/// 'name' = Name
const OS_TYPE_NAME: &[u8; 4] = b"name";


#[derive(Debug, PartialEq, Fail)]
pub enum ImageResourcesDescriptorError {
    #[fail(
    display = r#"Invalid TypeOS field."#
    )]
    InvalidTypeOS {},
    #[fail(
    display = r#"Invalid unit name."#
    )]
    InvalidUnitName {},
}

impl DescriptorStructure {
    fn read_descriptor_structure(cursor: &mut PsdCursor) -> Result<DescriptorStructure, Error> {
        let name = cursor.read_unicode_string_padding(1)?;
        let class_id = DescriptorStructure::read_key_length(cursor)?.to_vec();
        let fields = DescriptorStructure::read_fields(cursor, false)?;

        Ok(DescriptorStructure {
            name,
            fields,
            class_id,
        })
    }

    fn read_fields(cursor: &mut PsdCursor, sub_list: bool) -> Result<HashMap<String, DescriptorField>, Error> {
        let count = cursor.read_u32()?;
        let mut m = HashMap::with_capacity(count as usize);
        println!("count: {:?}", count);

        for n in 0..count {
            let key = DescriptorStructure::read_key_length(cursor)?;
            let key = String::from_utf8(key.to_vec())?;

            m.insert(key, DescriptorStructure::read_descriptor_field(cursor)?);
        }

        Ok(m)
    }

    fn read_list(cursor: &mut PsdCursor, sub_list: bool) -> Result<Vec<DescriptorField>, Error> {
        let count = cursor.read_u32()?;
        let mut vec = Vec::with_capacity(count as usize);
        println!("count: {:?}", count);

        for n in 0..count {
            let field = DescriptorStructure::read_descriptor_field(cursor)?;
            vec.push(field);
        }

        Ok(vec)
    }

    fn read_descriptor_field(cursor: &mut PsdCursor) -> Result<DescriptorField, Error> {
        let mut os_type = [0; 4];
        os_type.copy_from_slice(cursor.read_4()?);
        println!("{:?}", String::from_utf8(os_type.to_vec())?);

        let r: DescriptorField = match &os_type {
            OS_TYPE_REFERENCE => DescriptorField::Reference(DescriptorStructure::read_reference_structure(cursor)?),
            OS_TYPE_DESCRIPTOR => DescriptorField::Descriptor(DescriptorStructure::read_descriptor_structure(cursor)?),
            OS_TYPE_LIST => DescriptorField::List(DescriptorStructure::read_list_structure(cursor)?),
            OS_TYPE_DOUBLE => DescriptorField::Double(cursor.read_f64()?),
            OS_TYPE_UNIT_FLOAT => DescriptorField::UnitFloat(DescriptorStructure::read_unit_float(cursor)?),
            OS_TYPE_TEXT => DescriptorField::String(cursor.read_unicode_string_padding(1)?),
            OS_TYPE_ENUMERATED => DescriptorField::EnumeratedDescriptor(DescriptorStructure::read_enumerated_descriptor(cursor)?),
            OS_TYPE_LARGE_INTEGER => DescriptorField::LargeInteger(cursor.read_i64()?),
            OS_TYPE_INTEGER => DescriptorField::Integer(cursor.read_i32()?),
            OS_TYPE_BOOL => DescriptorField::Boolean(cursor.read_u8()? > 0),
            OS_TYPE_GLOBAL_OBJECT => DescriptorField::Descriptor(DescriptorStructure::read_descriptor_structure(cursor)?),
            OS_TYPE_CLASS => DescriptorField::Class(DescriptorStructure::read_class_structure(cursor)?),
            OS_TYPE_CLASS2 => DescriptorField::Class(DescriptorStructure::read_class_structure(cursor)?),
            OS_TYPE_ALIAS => DescriptorField::Alias(DescriptorStructure::read_alias_structure(cursor)?),
            OS_TYPE_RAW_DATA => DescriptorField::RawData(DescriptorStructure::read_raw_data(cursor)?),
            _ => return Err(ImageResourcesDescriptorError::InvalidTypeOS {}.into()),
        };

        Ok(r)
    }

    fn read_reference_structure(cursor: &mut PsdCursor) -> Result<Vec<DescriptorField>, Error> {
        let count = cursor.read_u32()?;
        let mut vec = Vec::with_capacity(count as usize);

        for n in 0..count {
            DescriptorStructure::read_key_length(cursor)?;

            let mut os_type = [0; 4];
            os_type.copy_from_slice(cursor.read_4()?);
            vec.push(match &os_type {
                OS_TYPE_PROPERTY => DescriptorField::Property(DescriptorStructure::read_property_structure(cursor)?),
                OS_TYPE_CLASS3 => DescriptorField::Class(DescriptorStructure::read_class_structure(cursor)?),
                OS_TYPE_ENUMERATED_REFERENCE => DescriptorField::EnumeratedReference(DescriptorStructure::read_enumerated_reference(cursor)?),
                OS_TYPE_OFFSET => DescriptorField::Offset(DescriptorStructure::read_offset_structure(cursor)?),
                OS_TYPE_IDENTIFIER => DescriptorField::Identifier(cursor.read_i32()?),
                OS_TYPE_INDEX => DescriptorField::Index(cursor.read_i32()?),
                OS_TYPE_NAME => DescriptorField::Name(DescriptorStructure::read_name(cursor)?),
                _ => return Err(ImageResourcesDescriptorError::InvalidTypeOS {}.into()),
            });
        }

        Ok(vec)
    }

    fn read_property_structure(cursor: &mut PsdCursor) -> Result<PropertyStructure, Error> {
        let name = cursor.read_unicode_string()?;
        let class_id = DescriptorStructure::read_key_length(cursor)?.to_vec();
        let key_id = DescriptorStructure::read_key_length(cursor)?.to_vec();

        Ok(PropertyStructure {
            name,
            class_id,
            key_id,
        })
    }

    fn read_unit_float(cursor: &mut PsdCursor) -> Result<UnitFloatStructure, Error> {
        let mut unit_float = [0; 4];
        unit_float.copy_from_slice(cursor.read_4()?);

        Ok(match &unit_float {
            UNIT_FLOAT_ANGLE => UnitFloatStructure::Angle(cursor.read_f64()?),
            UNIT_FLOAT_DENSITY => UnitFloatStructure::Density(cursor.read_f64()?),
            UNIT_FLOAT_DISTANCE => UnitFloatStructure::Distance(cursor.read_f64()?),
            UNIT_FLOAT_NONE => UnitFloatStructure::None,
            UNIT_FLOAT_PERCENT => UnitFloatStructure::Percent(cursor.read_f64()?),
            UNIT_FLOAT_PIXELS => UnitFloatStructure::Pixels(cursor.read_f64()?),
            _ => return Err(ImageResourcesDescriptorError::InvalidUnitName {}.into())
        })
    }

    fn read_class_structure(cursor: &mut PsdCursor) -> Result<ClassStructure, Error> {
        let name = cursor.read_unicode_string()?;
        let class_id = DescriptorStructure::read_key_length(cursor)?.to_vec();

        Ok(ClassStructure {
            name,
            class_id,
        })
    }

    fn read_enumerated_reference(cursor: &mut PsdCursor) -> Result<EnumeratedReference, Error> {
        let name = cursor.read_unicode_string()?;
        let class_id = DescriptorStructure::read_key_length(cursor)?.to_vec();
        let key_id = DescriptorStructure::read_key_length(cursor)?.to_vec();
        let enum_field = DescriptorStructure::read_key_length(cursor)?.to_vec();

        Ok(EnumeratedReference {
            name,
            class_id,
            key_id,
            enum_field,
        })
    }

    fn read_offset_structure(cursor: &mut PsdCursor) -> Result<OffsetStructure, Error> {
        let name = cursor.read_unicode_string()?;
        let class_id = DescriptorStructure::read_key_length(cursor)?.to_vec();
        let offset = cursor.read_u32()?;

        Ok(OffsetStructure {
            name,
            class_id,
            offset,
        })
    }

    fn read_alias_structure(cursor: &mut PsdCursor) -> Result<AliasStructure, Error> {
        let length = cursor.read_u32()?;
        let data = cursor.read(length)?.to_vec();

        Ok(AliasStructure {
            data
        })
    }

    fn read_list_structure(cursor: &mut PsdCursor) -> Result<Vec<DescriptorField>, Error> {
        DescriptorStructure::read_list(cursor, true)
    }

    fn read_enumerated_descriptor(cursor: &mut PsdCursor) -> Result<EnumeratedDescriptor, Error> {
        let type_field = DescriptorStructure::read_key_length(cursor)?.to_vec();
        let enum_field = DescriptorStructure::read_key_length(cursor)?.to_vec();

        Ok(EnumeratedDescriptor {
            type_field,
            enum_field,
        })
    }

    fn read_raw_data(cursor: &mut PsdCursor) -> Result<Vec<u8>, Error> {
        let length = cursor.read_u32()?;
        Ok(cursor.read(length)?.to_vec())
    }

    // Note: this structure is not documented
    fn read_name(cursor: &mut PsdCursor) -> Result<NameStructure, Error> {
        let name = cursor.read_unicode_string()?;
        let class_id = DescriptorStructure::read_key_length(cursor)?.to_vec();
        let value = cursor.read_unicode_string()?;

        Ok(NameStructure {
            name,
            class_id,
            value,
        })
    }

    fn read_key_length<'a>(cursor: &'a mut PsdCursor) -> Result<&'a [u8], Error> {
        let length = cursor.read_u32()?;
        let length = if length > 0 {
            length
        } else {
            4
        };

        let key = cursor.read(length)?;
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_descriptor() {
        let bytes = include_bytes!("../../tests/fixtures/descriptors/0.dat");
        let mut cursor = PsdCursor::new(bytes);
        DescriptorStructure::read_descriptor_structure(&mut cursor).unwrap();
    }

    #[test]
    fn check_descriptor2() {
        let bytes = include_bytes!("../../tests/fixtures/descriptors/1.dat");
        let mut cursor = PsdCursor::new(bytes);
        DescriptorStructure::read_descriptor_structure(&mut cursor).unwrap();
    }

    #[test]
    fn check_slices() {
        let bytes = include_bytes!("../../tests/fixtures/slices/slices_0.dat");
        ImageResourcesSection::read_slice_block(bytes).unwrap();
    }
}


fn get_name(resource_id: i16) -> String {
    let s = match resource_id {
        1000 => "(Obsolete--Photoshop 2.0 only ) Contains five 2-byte values: number of channels, rows, columns, depth, and mode",
        1001 => "Macintosh print manager print info record",
        1002 => "Macintosh page format information. No longer read by Photoshop. (Obsolete)",
        1003 => "(Obsolete--Photoshop 2.0 only ) Indexed color table",
        1005 => "ResolutionInfo structure. See Appendix A in Photoshop API Guide.pdf.",
        1006 => "Names of the alpha channels as a series of Pascal strings.",
        1007 => "(Obsolete) See ID 1077DisplayInfo structure. See Appendix A in Photoshop API Guide.pdf.",
        1008 => "The caption as a Pascal string.",
        1009 => "Border information. Contains a fixed number (2 bytes real, 2 bytes fraction) for the border width, and 2 bytes for border units (1 = inches, 2 = cm, 3 = points, 4 = picas, 5 = columns).",
        1010 => "Background color. See See Color structure.",
        1011 => "Print flags. A series of one-byte boolean values (see Page Setup dialog): labels, crop marks, color bars, registration marks, negative, flip, interpolate, caption, print flags.",
        1012 => "Grayscale and multichannel halftoning information",
        1013 => "Color halftoning information",
        1014 => "Duotone halftoning information",
        1015 => "Grayscale and multichannel transfer function",
        1016 => "Color transfer functions",
        1017 => "Duotone transfer functions",
        1018 => "Duotone image information",
        1019 => "Two bytes for the effective black and white values for the dot range",
        1020 => "(Obsolete)",
        1021 => "EPS options",
        1022 => "Quick Mask information. 2 bytes containing Quick Mask channel ID; 1- byte boolean indicating whether the mask was initially empty.",
        1023 => "(Obsolete)",
        1024 => "Layer state information. 2 bytes containing the index of target layer (0 = bottom layer).",
        1025 => "Working path (not saved). See See Path resource format.",
        1026 => "Layers group information. 2 bytes per layer containing a group ID for the dragging groups. Layers in a group have the same group ID.",
        1027 => "(Obsolete)",
        1028 => "IPTC-NAA record. Contains the File Info... information. See the documentation in the IPTC folder of the Documentation folder.",
        1029 => "Image mode for raw format files",
        1030 => "JPEG quality. Private.",
        1032 => "(Photoshop 4.0) Grid and guides information. See See Grid and guides resource format.",
        1033 => "(Photoshop 4.0) Thumbnail resource for Photoshop 4.0 only. See See Thumbnail resource format.",
        1034 => "(Photoshop 4.0) Copyright flag. Boolean indicating whether image is copyrighted. Can be set via Property suite or by user in File Info...",
        1035 => "(Photoshop 4.0) URL. Handle of a text string with uniform resource locator. Can be set via Property suite or by user in File Info...",
        1036 => "(Photoshop 5.0) Thumbnail resource (supersedes resource 1033). See See Thumbnail resource format.",
        1037 => "(Photoshop 5.0) Global Angle. 4 bytes that contain an integer between 0 and 359, which is the global lighting angle for effects layer. If not present, assumed to be 30.",
        1038 => "(Obsolete) See ID 1073 below. (Photoshop 5.0) Color samplers resource. See See Color samplers resource format.",
        1039 => "(Photoshop 5.0) ICC Profile. The raw bytes of an ICC (International Color Consortium) format profile. See ICC1v42_2006-05.pdf in the Documentation folder and icProfileHeader.h in Sample Code\\Common\\Includes .",
        1040 => "(Photoshop 5.0) Watermark. One byte.",
        1041 => "(Photoshop 5.0) ICC Untagged Profile. 1 byte that disables any assumed profile handling when opening the file. 1 = intentionally untagged.",
        1042 => "(Photoshop 5.0) Effects visible. 1-byte global flag to show/hide all the effects layer. Only present when they are hidden.",
        1043 => "(Photoshop 5.0) Spot Halftone. 4 bytes for version, 4 bytes for length, and the variable length data.",
        1044 => "(Photoshop 5.0) Document-specific IDs seed number. 4 bytes: Base value, starting at which layer IDs will be generated (or a greater value if existing IDs already exceed it). Its purpose is to avoid the case where we add layers, flatten, save, open, and then add more layers that end up with the same IDs as the first set.",
        1045 => "(Photoshop 5.0) Unicode Alpha Names. Unicode string",
        1046 => "(Photoshop 6.0) Indexed Color Table Count. 2 bytes for the number of colors in table that are actually defined",
        1047 => "(Photoshop 6.0) Transparency Index. 2 bytes for the index of transparent color, if any.",
        1049 => "(Photoshop 6.0) Global Altitude. 4 byte entry for altitude",
        1050 => "(Photoshop 6.0) Slices. See See Slices resource format.",
        1051 => "(Photoshop 6.0) Workflow URL. Unicode string",
        1052 => "(Photoshop 6.0) Jump To XPEP. 2 bytes major version, 2 bytes minor version, 4 bytes count. Following is repeated for count: 4 bytes block size, 4 bytes key, if key = 'jtDd' , then next is a Boolean for the dirty flag; otherwise it's a 4 byte entry for the mod date.",
        1053 => "(Photoshop 6.0) Alpha Identifiers. 4 bytes of length, followed by 4 bytes each for every alpha identifier.",
        1054 => "(Photoshop 6.0) URL List. 4 byte count of URLs, followed by 4 byte long, 4 byte ID, and Unicode string for each count.",
        1057 => "(Photoshop 6.0) Version Info. 4 bytes version, 1 byte hasRealMergedData , Unicode string: writer name, Unicode string: reader name, 4 bytes file version.",
        1058 => "(Photoshop 7.0) EXIF data 1. See http://www.kodak.com/global/plugins/acrobat/en/service/digCam/exifStandard2.pdf",
        1059 => "(Photoshop 7.0) EXIF data 3. See http://www.kodak.com/global/plugins/acrobat/en/service/digCam/exifStandard2.pdf",
        1060 => "(Photoshop 7.0) XMP metadata. File info as XML description. See http://www.adobe.com/devnet/xmp/",
        1061 => "(Photoshop 7.0) Caption digest. 16 bytes: RSA Data Security, MD5 message-digest algorithm",
        1062 => "(Photoshop 7.0) Print scale. 2 bytes style (0 = centered, 1 = size to fit, 2 = user defined). 4 bytes x location (floating point). 4 bytes y location (floating point). 4 bytes scale (floating point)",
        1064 => "(Photoshop CS) Pixel Aspect Ratio. 4 bytes (version = 1 or 2), 8 bytes double, x / y of a pixel. Version 2, attempting to correct values for NTSC and PAL, previously off by a factor of approx. 5%.",
        1065 => "(Photoshop CS) Layer Comps. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure)",
        1066 => "(Photoshop CS) Alternate Duotone Colors. 2 bytes (version = 1), 2 bytes count, following is repeated for each count: [ Color: 2 bytes for space followed by 4 * 2 byte color component ], following this is another 2 byte count, usually 256, followed by Lab colors one byte each for L, a, b. This resource is not read or used by Photoshop.",
        1067 => "(Photoshop CS)Alternate Spot Colors. 2 bytes (version = 1), 2 bytes channel count, following is repeated for each count: 4 bytes channel ID, Color: 2 bytes for space followed by 4 * 2 byte color component. This resource is not read or used by Photoshop.",
        1069 => "(Photoshop CS2) Layer Selection ID(s). 2 bytes count, following is repeated for each count: 4 bytes layer ID",
        1070 => "(Photoshop CS2) HDR Toning information",
        1071 => "(Photoshop CS2) Print info",
        1072 => "(Photoshop CS2) Layer Group(s) Enabled ID. 1 byte for each layer in the document, repeated by length of the resource. NOTE: Layer groups have start and end markers",
        1073 => "(Photoshop CS3) Color samplers resource. Also see ID 1038 for old format. See See Color samplers resource format.",
        1074 => "(Photoshop CS3) Measurement Scale. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure)",
        1075 => "(Photoshop CS3) Timeline Information. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure)",
        1076 => "(Photoshop CS3) Sheet Disclosure. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure)",
        1077 => "(Photoshop CS3) DisplayInfo structure to support floating point clors. Also see ID 1007. See Appendix A in Photoshop API Guide.pdf .",
        1078 => "(Photoshop CS3) Onion Skins. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure)",
        1080 => "(Photoshop CS4) Count Information. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure) Information about the count in the document. See the Count Tool.",
        1082 => "(Photoshop CS5) Print Information. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure) Information about the current print settings in the document. The color management options.",
        1083 => "(Photoshop CS5) Print Style. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure) Information about the current print style in the document. The printing marks, labels, ornaments, etc.",
        1084 => "(Photoshop CS5) Macintosh NSPrintInfo. Variable OS specific info for Macintosh. NSPrintInfo. It is recommened that you do not interpret or use this data.",
        1085 => "(Photoshop CS5) Windows DEVMODE. Variable OS specific info for Windows. DEVMODE. It is recommened that you do not interpret or use this data.",
        1086 => "(Photoshop CS6) Auto Save File Path. Unicode string. It is recommened that you do not interpret or use this data.",
        1087 => "(Photoshop CS6) Auto Save Format. Unicode string. It is recommened that you do not interpret or use this data.",
        1088 => "(Photoshop CC) Path Selection State. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure) Information about the current path selection state.",
        2999 => "Name of clipping path. See See Path resource format.",
        3000 => "(Photoshop CC) Origin Path Info. 4 bytes (descriptor version = 16), Descriptor (see See Descriptor structure) Information about the origin path data.",
        7000 => "Image Ready variables. XML representation of variables definition",
        7001 => "Image Ready data sets",
        7002 => "Image Ready default selected state",
        7003 => "Image Ready 7 rollover expanded state",
        7004 => "Image Ready rollover expanded state",
        7005 => "Image Ready save layer settings",
        7006 => "Image Ready version",
        8000 => "(Photoshop CS3) Lightroom workflow, if present the document is in the middle of a Lightroom workflow.",
        10000 => "Print flags information. 2 bytes version ( = 1), 1 byte center crop marks, 1 byte ( = 0), 4 bytes bleed width value, 2 bytes bleed width scale.",
        _ => "Unknown"
    };
    String::from(s)
}