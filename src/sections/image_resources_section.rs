use std::collections::HashMap;
use std::ops::Range;

use thiserror::Error;

pub use crate::sections::image_resources_section::image_resource::ImageResource;
use crate::sections::image_resources_section::image_resource::SlicesImageResource;
use crate::sections::PsdCursor;

const EXPECTED_RESOURCE_BLOCK_SIGNATURE: [u8; 4] = [56, 66, 73, 77];
const EXPECTED_DESCRIPTOR_VERSION: u32 = 16;
const RESOURCE_SLICES_INFO: i16 = 1050;

mod image_resource;

struct ImageResourcesBlock {
    resource_id: i16,
    name: String,
    data_range: Range<usize>,
}

#[derive(Debug)]
pub struct ImageResourcesSection {
    pub(crate) resources: Vec<ImageResource>,
}

/// Represents an malformed resource block
#[derive(Debug, PartialEq, Error)]
pub enum ImageResourcesSectionError {
    #[error(
        r#"The first four bytes (indices 0-3) must always equal [56, 66, 73, 77],
         which in string form is '8BIM'."#
    )]
    InvalidSignature {},

    #[error("Invalid resource descriptor: {0}")]
    InvalidResource(ImageResourcesDescriptorError),
}

impl ImageResourcesSection {
    pub fn from_bytes(bytes: &[u8]) -> Result<ImageResourcesSection, ImageResourcesSectionError> {
        let mut cursor = PsdCursor::new(bytes);

        let mut resources = vec![];

        let length = cursor.read_u32() as u64;

        while cursor.position() < length {
            let block = ImageResourcesSection::read_resource_block(&mut cursor)?;

            let rid = block.resource_id;
            match rid {
                _ if rid == RESOURCE_SLICES_INFO => {
                    let slices_image_resource = ImageResourcesSection::read_slice_block(
                        &cursor.get_ref()[block.data_range],
                    )
                    .map_err(ImageResourcesSectionError::InvalidResource)?;
                    resources.push(ImageResource::Slices(slices_image_resource));
                }
                _ => {}
            }
        }

        assert_eq!(cursor.position(), length + 4);

        Ok(ImageResourcesSection { resources })
    }

    /// +----------+--------------------------------------------------------------------------------------------------------------------+
    /// |  Length  |                                                    Description                                                     |
    /// +----------+--------------------------------------------------------------------------------------------------------------------+
    /// | 4        | Signature: '8BIM'                                                                                                  |
    /// | 2        | Unique identifier for the resource. Image resource IDs contains a list of resource IDs used by Photoshop.          |
    /// | Variable | Name: Pascal string, padded to make the size even (a null name consists of two bytes of 0)                         |
    /// | 4        | Actual size of resource data that follows                                                                          |
    /// | Variable | The resource data, described in the sections on the individual resource types. It is padded to make the size even. |
    /// +----------+--------------------------------------------------------------------------------------------------------------------+
    fn read_resource_block(
        cursor: &mut PsdCursor,
    ) -> Result<ImageResourcesBlock, ImageResourcesSectionError> {
        // First four bytes must be '8BIM'
        let signature = cursor.read_4();
        if signature != EXPECTED_RESOURCE_BLOCK_SIGNATURE {
            return Err(ImageResourcesSectionError::InvalidSignature {});
        }

        let resource_id = cursor.read_i16();
        let name = cursor.read_pascal_string();

        let data_len = cursor.read_u32();
        let pos = cursor.position() as usize;
        // Note: data length is padded to even.
        let data_len = data_len + data_len % 2;
        let data_range = Range {
            start: pos,
            end: pos + data_len as usize,
        };
        cursor.read(data_len);

        Ok(ImageResourcesBlock {
            resource_id,
            name,
            data_range,
        })
    }

    /// Slice header for version 6
    ///
    /// +----------+--------------------------------------------------------------------------------------+
    /// |  Length  |                                     Description                                      |
    /// +----------+--------------------------------------------------------------------------------------+
    /// | 4        | Version ( = 6)                                                                       |
    /// | 4 * 4    | Bounding rectangle for all of the slices: top, left, bottom, right of all the slices |
    /// | Variable | Name of group of slices: Unicode string                                              |
    /// | 4        | Number of slices to follow. See Slices resource block in the next table              |
    /// +----------+--------------------------------------------------------------------------------------+
    fn read_slice_block(
        bytes: &[u8],
    ) -> Result<SlicesImageResource, ImageResourcesDescriptorError> {
        let mut cursor = PsdCursor::new(bytes);

        let version = cursor.read_i32();
        if version == 6 {
            let _top = cursor.read_i32();
            let _left = cursor.read_i32();
            let _bottom = cursor.read_i32();
            let _right = cursor.read_i32();

            let group_of_slices_name = cursor.read_unicode_string_padding(1);

            let number_of_slices = cursor.read_u32();

            let mut descriptors = Vec::new();

            for _ in 0..number_of_slices {
                match ImageResourcesSection::read_slice_body(&mut cursor)? {
                    Some(v) => descriptors.push(v),
                    None => {}
                }
            }

            return Ok(SlicesImageResource {
                name: group_of_slices_name,
                descriptors,
            });
        }
        if version == 7 || version == 8 {
            let descriptor_version = cursor.read_i32();
            if descriptor_version != 16 {
                unimplemented!(
                    "Only the version 16 (descriptors) resource format for slices is currently supported"
                );
            }
            let descriptor = DescriptorStructure::read_descriptor_structure(&mut cursor)?;
            return Ok(SlicesImageResource {
                name: descriptor.name.clone(),
                descriptors: vec![descriptor],
            });
        }
        unimplemented!("Slices resource format {version} is currently not supported");
    }

    /// Slices resource block
    ///
    /// +------------------------------------------------------+-----------------------------------------------+
    /// |                        Length                        |                  Description                  |
    /// +------------------------------------------------------+-----------------------------------------------+
    /// | 4                                                    | ID                                            |
    /// | 4                                                    | Group ID                                      |
    /// | 4                                                    | Origin                                        |
    /// | 4                                                    | Associated Layer ID                           |
    /// | Only present if Origin = 1                           |                                               |
    /// | Variable                                             | Name: Unicode string                          |
    /// | 4                                                    | Type                                          |
    /// | 4 * 4                                                | Left, top, right, bottom positions            |
    /// | Variable                                             | URL: Unicode string                           |
    /// | Variable                                             | Target: Unicode string                        |
    /// | Variable                                             | Message: Unicode string                       |
    /// | Variable                                             | Alt Tag: Unicode string                       |
    /// | 1                                                    | Cell text is HTML: Boolean                    |
    /// | Variable                                             | Cell text: Unicode string                     |
    /// | 4                                                    | Horizontal alignment                          |
    /// | 4                                                    | Vertical alignment                            |
    /// | 1                                                    | Alpha color                                   |
    /// | 1                                                    | Red                                           |
    /// | 1                                                    | Green                                         |
    /// | 1                                                    | Blue                                          |
    /// | Additional data as length allows. See comment above. |                                               |
    /// | 4                                                    | Descriptor version ( = 16 for Photoshop 6.0). |
    /// | Variable                                             | Descriptor (see See Descriptor structure)     |
    /// +------------------------------------------------------+-----------------------------------------------+
    fn read_slice_body(
        cursor: &mut PsdCursor,
    ) -> Result<Option<DescriptorStructure>, ImageResourcesDescriptorError> {
        let _slice_id = cursor.read_i32();
        let _group_id = cursor.read_i32();
        let origin = cursor.read_i32();

        // if origin = 1, Associated Layer ID is present
        if origin == 1 {
            cursor.read_i32();
        }

        let _name = cursor.read_unicode_string_padding(1);

        let _type = cursor.read_i32();

        let _top = cursor.read_i32();
        let _left = cursor.read_i32();
        let _bottom = cursor.read_i32();
        let _right = cursor.read_i32();

        let _url = cursor.read_unicode_string_padding(1);

        let _target = cursor.read_unicode_string_padding(1);

        let _message = cursor.read_unicode_string_padding(1);

        let _alt_tag = cursor.read_unicode_string_padding(1);

        let _cell_text_html = cursor.read_1();
        let _cell_text = cursor.read_unicode_string_padding(1);

        let _horizontal_alignment = cursor.read_i32();
        let _vertical_alignment = cursor.read_i32();
        let _argb_color = cursor.read_i32();

        let pos = cursor.position();
        let descriptor_version = cursor.peek_u32();

        Ok(if descriptor_version == EXPECTED_DESCRIPTOR_VERSION {
            cursor.read_4();

            let descriptor = DescriptorStructure::read_descriptor_structure(cursor)?;
            if descriptor.class_id.as_slice() == [0, 0, 0, 0] {
                cursor.seek(pos);
            }

            Some(descriptor)
        } else {
            None
        })
    }
}

/// +-------------------------------------------------------+--------------------------------------------------------------------------------------------+
/// |                        Length                         |                                        Description                                         |
/// +-------------------------------------------------------+--------------------------------------------------------------------------------------------+
/// | Variable                                              | Unicode string: name from classID                                                          |
/// | Variable                                              | classID: 4 bytes (length), followed either by string or (if length is zero) 4-byte classID |
/// | 4                                                     | Number of items in descriptor                                                              |
/// | The following is repeated for each item in descriptor |                                                                                            |
/// | Variable                                              | Key: 4 bytes ( length) followed either by string or (if length is zero) 4-byte key         |
/// | 4                                                     | Type: OSType key                                                                           |
/// |                                                       | 'obj ' = Reference                                                                         |
/// |                                                       | 'Objc' = Descriptor                                                                        |
/// |                                                       | 'VlLs' = List                                                                              |
/// |                                                       | 'doub' = Double                                                                            |
/// |                                                       | 'UntF' = Unit float                                                                        |
/// |                                                       | 'TEXT' = String                                                                            |
/// |                                                       | 'enum' = Enumerated                                                                        |
/// |                                                       | 'long' = Integer                                                                           |
/// |                                                       | 'comp' = Large Integer                                                                     |
/// |                                                       | 'bool' = Boolean                                                                           |
/// |                                                       | 'GlbO' = GlobalObject same as Descriptor                                                   |
/// |                                                       | 'type' = Class                                                                             |
/// |                                                       | 'GlbC' = Class                                                                             |
/// |                                                       | 'alis' = Alias                                                                             |
/// |                                                       | 'tdta' = Raw Data                                                                          |
/// | Variable                                              | Item type: see the tables below for each possible type                                     |
/// +-------------------------------------------------------+--------------------------------------------------------------------------------------------+
#[derive(Debug)]
pub struct DescriptorStructure {
    pub name: String,
    pub fields: HashMap<String, DescriptorField>,
    pub class_id: Vec<u8>,
}

/// One of
#[derive(Debug)]
pub enum DescriptorField {
    /// Descriptor as field
    Descriptor(DescriptorStructure),
    /// A list of special fields
    /// There are can be Property, Identifier, Index, Name fields
    Reference(Vec<DescriptorField>),
    /// Float field with unit
    UnitFloat(UnitFloatStructure),
    /// Double-precision floating-point number
    Double(f64),
    ///
    Class(ClassStructure),
    /// Text
    String(String),
    ///
    EnumeratedReference(EnumeratedReference),
    ///
    Offset(OffsetStructure),
    /// Boolean value
    Boolean(bool),
    ///
    Alias(AliasStructure),
    /// A list of fields
    List(Vec<DescriptorField>),
    /// 64bit integer number
    LargeInteger(i64),
    /// 32bit integer number
    Integer(i32),
    ///
    EnumeratedDescriptor(EnumeratedDescriptor),
    /// Raw bytes data
    RawData(Vec<u8>),

    /// Only Reference fields
    ///
    ///
    Property(PropertyStructure),
    ///
    Identifier(i32),
    ///
    Index(i32),
    ///
    Name(NameStructure),
}

/// +----------+--------------------------------------------------------------------------------------------+
/// |  Length  |                                        Description                                         |
/// +----------+--------------------------------------------------------------------------------------------+
/// | Variable | Unicode string: name from classID                                                          |
/// | Variable | classID: 4 bytes (length), followed either by string or (if length is zero) 4-byte classID |
/// | Variable | KeyID: 4 bytes (length), followed either by string or (if length is zero) 4-byte keyID     |
/// +----------+--------------------------------------------------------------------------------------------+
#[derive(Debug)]
pub struct PropertyStructure {
    pub name: String,
    pub class_id: Vec<u8>,
    pub key_id: Vec<u8>,
}

/// +------------------------------------+--------------------------------------------------------+
/// |               Length               |                      Description                       |
/// +------------------------------------+--------------------------------------------------------+
/// | 4                                  | Units the following value is in. One of the following: |
/// |                                    | '#Ang' = angle: base degrees                           |
/// |                                    | '#Rsl' = density: base per inch                        |
/// |                                    | '#Rlt' = distance: base 72ppi                          |
/// |                                    | '#Nne' = none: coerced.                                |
/// |                                    | '#Prc'= percent: unit value                            |
/// |                                    | '#Pxl' = pixels: tagged unit value                     |
/// | 8                                  | Actual value (double)                                  |
/// +------------------------------------+--------------------------------------------------------+
#[derive(Debug)]
pub enum UnitFloatStructure {
    /// Base degrees
    Angle(f64),
    /// Base per inch
    Density(f64),
    /// Base 72ppi
    Distance(f64),
    /// Base coerced
    None,
    /// Unit value
    Percent(f64),
    /// Tagged unit value
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

/// +----------+--------------------------------------------------------------------------------------------+
/// |  Length  |                                        Description                                         |
/// +----------+--------------------------------------------------------------------------------------------+
/// | Variable | Unicode string: name from classID                                                          |
/// | Variable | ClassID: 4 bytes (length), followed either by string or (if length is zero) 4-byte classID |
/// +----------+--------------------------------------------------------------------------------------------+
#[derive(Debug)]
pub struct ClassStructure {
    pub name: String,
    pub class_id: Vec<u8>,
}

/// +----------+--------------------------------------------------------------------------------------------+
/// |  Length  |                                        Description                                         |
/// +----------+--------------------------------------------------------------------------------------------+
/// | Variable | Unicode string: name from ClassID.                                                         |
/// | Variable | ClassID: 4 bytes (length), followed either by string or (if length is zero) 4-byte classID |
/// | Variable | TypeID: 4 bytes (length), followed either by string or (if length is zero) 4-byte typeID   |
/// | Variable | enum: 4 bytes (length), followed either by string or (if length is zero) 4-byte enum       |
/// +----------+--------------------------------------------------------------------------------------------+
#[derive(Debug)]
pub struct EnumeratedReference {
    pub name: String,
    pub class_id: Vec<u8>,
    pub key_id: Vec<u8>,
    pub enum_field: Vec<u8>,
}

/// +----------+--------------------------------------------------------------------------------------------+
/// |  Length  |                                        Description                                         |
/// +----------+--------------------------------------------------------------------------------------------+
/// | Variable | Unicode string: name from ClassID                                                          |
/// | Variable | ClassID: 4 bytes (length), followed either by string or (if length is zero) 4-byte classID |
/// | 4        | Value of the offset                                                                        |
/// +----------+--------------------------------------------------------------------------------------------+
#[derive(Debug)]
pub struct OffsetStructure {
    pub name: String,
    pub class_id: Vec<u8>,
    pub offset: u32,
}

/// +----------+--------------------------------------------------------------------------+
/// |  Length  |                               Description                                |
/// +----------+--------------------------------------------------------------------------+
/// | 4        | Length of data to follow                                                 |
/// | Variable | FSSpec for Macintosh or a handle to a string to the full path on Windows |
/// +----------+--------------------------------------------------------------------------+
#[derive(Debug)]
pub struct AliasStructure {
    pub data: Vec<u8>,
}

/// +----------+----------------------------------------------------------------------------------------+
/// |  Length  |                                      Description                                       |
/// +----------+----------------------------------------------------------------------------------------+
/// | Variable | Type: 4 bytes (length), followed either by string or (if length is zero) 4-byte typeID |
/// | Variable | Enum: 4 bytes (length), followed either by string or (if length is zero) 4-byte enum   |
/// +----------+----------------------------------------------------------------------------------------+
#[derive(Debug)]
pub struct EnumeratedDescriptor {
    pub type_field: Vec<u8>,
    pub enum_field: Vec<u8>,
}

/// NOTE: This struct is not documented in the specification
/// So it's based on https://github.com/psd-tools/psd-tools/blob/master/src/psd_tools/psd/descriptor.py#L691
///
/// +----------+--------------------------------------------------------------------------------------------+
/// |  Length  |                                        Description                                         |
/// +----------+--------------------------------------------------------------------------------------------+
/// | Variable | Unicode string: name from ClassID                                                          |
/// | Variable | ClassID: 4 bytes (length), followed either by string or (if length is zero) 4-byte classID |
/// | Variable | Unicode string: value                                                                      |
/// +----------+--------------------------------------------------------------------------------------------+
#[derive(Debug)]
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

#[derive(Debug, PartialEq, Error)]
pub enum ImageResourcesDescriptorError {
    #[error(r#"Invalid TypeOS field."#)]
    InvalidTypeOS {},
    #[error(r#"Invalid unit name."#)]
    InvalidUnitName {},
}

impl DescriptorStructure {
    fn read_descriptor_structure(
        cursor: &mut PsdCursor,
    ) -> Result<DescriptorStructure, ImageResourcesDescriptorError> {
        let name = cursor.read_unicode_string_padding(1);
        let class_id = DescriptorStructure::read_key_length(cursor).to_vec();
        let fields = DescriptorStructure::read_fields(cursor, false)?;

        Ok(DescriptorStructure {
            name,
            fields,
            class_id,
        })
    }

    fn read_fields(
        cursor: &mut PsdCursor,
        sub_list: bool,
    ) -> Result<HashMap<String, DescriptorField>, ImageResourcesDescriptorError> {
        let count = cursor.read_u32();
        let mut m = HashMap::with_capacity(count as usize);

        for n in 0..count {
            let key = DescriptorStructure::read_key_length(cursor);
            let key = String::from_utf8_lossy(key).into_owned();

            m.insert(key, DescriptorStructure::read_descriptor_field(cursor)?);
        }

        Ok(m)
    }

    fn read_list(
        cursor: &mut PsdCursor,
        sub_list: bool,
    ) -> Result<Vec<DescriptorField>, ImageResourcesDescriptorError> {
        let count = cursor.read_u32();
        let mut vec = Vec::with_capacity(count as usize);

        for n in 0..count {
            let field = DescriptorStructure::read_descriptor_field(cursor)?;
            vec.push(field);
        }

        Ok(vec)
    }

    fn read_descriptor_field(
        cursor: &mut PsdCursor,
    ) -> Result<DescriptorField, ImageResourcesDescriptorError> {
        let mut os_type = [0; 4];
        os_type.copy_from_slice(cursor.read_4());

        let r: DescriptorField = match &os_type {
            OS_TYPE_REFERENCE => {
                DescriptorField::Reference(DescriptorStructure::read_reference_structure(cursor)?)
            }
            OS_TYPE_DESCRIPTOR => {
                DescriptorField::Descriptor(DescriptorStructure::read_descriptor_structure(cursor)?)
            }
            OS_TYPE_LIST => {
                DescriptorField::List(DescriptorStructure::read_list_structure(cursor)?)
            }
            OS_TYPE_DOUBLE => DescriptorField::Double(cursor.read_f64()),
            OS_TYPE_UNIT_FLOAT => {
                DescriptorField::UnitFloat(DescriptorStructure::read_unit_float(cursor)?)
            }
            OS_TYPE_TEXT => DescriptorField::String(cursor.read_unicode_string_padding(1)),
            OS_TYPE_ENUMERATED => DescriptorField::EnumeratedDescriptor(
                DescriptorStructure::read_enumerated_descriptor(cursor),
            ),
            OS_TYPE_LARGE_INTEGER => DescriptorField::LargeInteger(cursor.read_i64()),
            OS_TYPE_INTEGER => DescriptorField::Integer(cursor.read_i32()),
            OS_TYPE_BOOL => DescriptorField::Boolean(cursor.read_u8() > 0),
            OS_TYPE_GLOBAL_OBJECT => {
                DescriptorField::Descriptor(DescriptorStructure::read_descriptor_structure(cursor)?)
            }
            OS_TYPE_CLASS => {
                DescriptorField::Class(DescriptorStructure::read_class_structure(cursor))
            }
            OS_TYPE_CLASS2 => {
                DescriptorField::Class(DescriptorStructure::read_class_structure(cursor))
            }
            OS_TYPE_ALIAS => {
                DescriptorField::Alias(DescriptorStructure::read_alias_structure(cursor))
            }
            OS_TYPE_RAW_DATA => {
                DescriptorField::RawData(DescriptorStructure::read_raw_data(cursor))
            }
            _ => return Err(ImageResourcesDescriptorError::InvalidTypeOS {}),
        };

        Ok(r)
    }

    /// +------------------------------------------------------+------------------------------------------------------------------+
    /// |                        Length                        |                           Description                            |
    /// +------------------------------------------------------+------------------------------------------------------------------+
    /// | 4                                                    | Number of items                                                  |
    /// | The following is repeated for each item in reference |                                                                  |
    /// | 4                                                    | OSType key for type to use:                                      |
    /// | 'prop' = Property                                    |                                                                  |
    /// | 'Clss' = Class                                       |                                                                  |
    /// | 'Enmr' = Enumerated Reference                        |                                                                  |
    /// | 'rele' = Offset                                      |                                                                  |
    /// | 'Idnt' = Identifier                                  |                                                                  |
    /// | 'indx' = Index                                       |                                                                  |
    /// | 'name' =Name                                         |                                                                  |
    /// | Variable                                             | Item type: see the tables below for each possible Reference type |
    /// +------------------------------------------------------+------------------------------------------------------------------+
    fn read_reference_structure(
        cursor: &mut PsdCursor,
    ) -> Result<Vec<DescriptorField>, ImageResourcesDescriptorError> {
        let count = cursor.read_u32();
        let mut vec = Vec::with_capacity(count as usize);

        for n in 0..count {
            DescriptorStructure::read_key_length(cursor);

            let mut os_type = [0; 4];
            os_type.copy_from_slice(cursor.read_4());
            vec.push(match &os_type {
                OS_TYPE_PROPERTY => {
                    DescriptorField::Property(DescriptorStructure::read_property_structure(cursor))
                }
                OS_TYPE_CLASS3 => {
                    DescriptorField::Class(DescriptorStructure::read_class_structure(cursor))
                }
                OS_TYPE_ENUMERATED_REFERENCE => DescriptorField::EnumeratedReference(
                    DescriptorStructure::read_enumerated_reference(cursor),
                ),
                OS_TYPE_OFFSET => {
                    DescriptorField::Offset(DescriptorStructure::read_offset_structure(cursor))
                }
                OS_TYPE_IDENTIFIER => DescriptorField::Identifier(cursor.read_i32()),
                OS_TYPE_INDEX => DescriptorField::Index(cursor.read_i32()),
                OS_TYPE_NAME => DescriptorField::Name(DescriptorStructure::read_name(cursor)),
                _ => return Err(ImageResourcesDescriptorError::InvalidTypeOS {}),
            });
        }

        Ok(vec)
    }

    fn read_property_structure(cursor: &mut PsdCursor) -> PropertyStructure {
        let name = cursor.read_unicode_string();
        let class_id = DescriptorStructure::read_key_length(cursor).to_vec();
        let key_id = DescriptorStructure::read_key_length(cursor).to_vec();

        PropertyStructure {
            name,
            class_id,
            key_id,
        }
    }

    fn read_unit_float(
        cursor: &mut PsdCursor,
    ) -> Result<UnitFloatStructure, ImageResourcesDescriptorError> {
        let mut unit_float = [0; 4];
        unit_float.copy_from_slice(cursor.read_4());

        Ok(match &unit_float {
            UNIT_FLOAT_ANGLE => UnitFloatStructure::Angle(cursor.read_f64()),
            UNIT_FLOAT_DENSITY => UnitFloatStructure::Density(cursor.read_f64()),
            UNIT_FLOAT_DISTANCE => UnitFloatStructure::Distance(cursor.read_f64()),
            UNIT_FLOAT_NONE => UnitFloatStructure::None,
            UNIT_FLOAT_PERCENT => UnitFloatStructure::Percent(cursor.read_f64()),
            UNIT_FLOAT_PIXELS => UnitFloatStructure::Pixels(cursor.read_f64()),
            _ => return Err(ImageResourcesDescriptorError::InvalidUnitName {}),
        })
    }

    fn read_class_structure(cursor: &mut PsdCursor) -> ClassStructure {
        let name = cursor.read_unicode_string();
        let class_id = DescriptorStructure::read_key_length(cursor).to_vec();

        ClassStructure { name, class_id }
    }

    fn read_enumerated_reference(cursor: &mut PsdCursor) -> EnumeratedReference {
        let name = cursor.read_unicode_string();
        let class_id = DescriptorStructure::read_key_length(cursor).to_vec();
        let key_id = DescriptorStructure::read_key_length(cursor).to_vec();
        let enum_field = DescriptorStructure::read_key_length(cursor).to_vec();

        EnumeratedReference {
            name,
            class_id,
            key_id,
            enum_field,
        }
    }

    fn read_offset_structure(cursor: &mut PsdCursor) -> OffsetStructure {
        let name = cursor.read_unicode_string();
        let class_id = DescriptorStructure::read_key_length(cursor).to_vec();
        let offset = cursor.read_u32();

        OffsetStructure {
            name,
            class_id,
            offset,
        }
    }

    fn read_alias_structure(cursor: &mut PsdCursor) -> AliasStructure {
        let length = cursor.read_u32();
        let data = cursor.read(length).to_vec();

        AliasStructure { data }
    }

    fn read_list_structure(
        cursor: &mut PsdCursor,
    ) -> Result<Vec<DescriptorField>, ImageResourcesDescriptorError> {
        DescriptorStructure::read_list(cursor, true)
    }

    fn read_enumerated_descriptor(cursor: &mut PsdCursor) -> EnumeratedDescriptor {
        let type_field = DescriptorStructure::read_key_length(cursor).to_vec();
        let enum_field = DescriptorStructure::read_key_length(cursor).to_vec();

        EnumeratedDescriptor {
            type_field,
            enum_field,
        }
    }

    fn read_raw_data(cursor: &mut PsdCursor) -> Vec<u8> {
        let length = cursor.read_u32();
        cursor.read(length).to_vec()
    }

    // Note: this structure is not documented
    fn read_name(cursor: &mut PsdCursor) -> NameStructure {
        let name = cursor.read_unicode_string();
        let class_id = DescriptorStructure::read_key_length(cursor).to_vec();
        let value = cursor.read_unicode_string();

        NameStructure {
            name,
            class_id,
            value,
        }
    }

    fn read_key_length<'a>(cursor: &'a mut PsdCursor) -> &'a [u8] {
        let length = cursor.read_u32();
        let length = if length > 0 { length } else { 4 };

        cursor.read(length)
    }
}
