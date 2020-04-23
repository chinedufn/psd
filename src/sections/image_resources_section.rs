use std::collections::HashMap;
use std::ops::Range;

use failure::{Error, Fail};

use crate::sections::PsdCursor;

const EXPECTED_RESOURCE_BLOCK_SIGNATURE: [u8; 4] = [56, 66, 73, 77];
const EXPECTED_DESCRIPTOR_VERSION: [u8; 4] = [0, 0, 0, 16];
const RESOURCE_SLICES_INFO: i16 = 1050;

struct ImageResourcesBlock {
    resource_id: i16,
    name: String,
    data_range: Range<usize>,
}

#[derive(Debug)]
pub struct ImageResourcesSection {
    pub descriptors: Option<Vec<DescriptorStructure>>,
}

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

        let mut descriptors = None;

        let length = cursor.read_u32()? as u64;

        while cursor.position() < length {
            let block = ImageResourcesSection::read_resource_block(&mut cursor)?;

            match block.resource_id {
                RESOURCE_SLICES_INFO => {
                    descriptors = Some(ImageResourcesSection::read_slice_block(
                        &cursor.get_ref()[block.data_range],
                    )?);
                }
                _ => {}
            }
        }

        assert_eq!(cursor.position(), length + 4);

        Ok(ImageResourcesSection { descriptors })
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
    fn read_resource_block(cursor: &mut PsdCursor) -> Result<ImageResourcesBlock, Error> {
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
    fn read_slice_block(bytes: &[u8]) -> Result<Vec<DescriptorStructure>, Error> {
        let mut cursor = PsdCursor::new(bytes);

        let version = cursor.read_i32()?;
        if version != 6 {
            unimplemented!(
                "Only the Adobe Photoshop 6.0 slices resource format is currently supported"
            );
        }

        let _top = cursor.read_i32()?;
        let _left = cursor.read_i32()?;
        let _bottom = cursor.read_i32()?;
        let _right = cursor.read_i32()?;

        let _group_of_slices_name = cursor.read_unicode_string_padding(1)?;

        let mut number_of_slices = cursor.read_u32()?;

        let mut vec = Vec::new();
        for _ in 0..number_of_slices {
            match ImageResourcesSection::read_slice_body(&mut cursor)? {
                Some(v) => vec.push(v),
                None => {}
            }
        }

        Ok(vec)
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
    fn read_slice_body(cursor: &mut PsdCursor) -> Result<Option<DescriptorStructure>, Error> {
        let slice_id = cursor.read_i32()?;
        let group_id = cursor.read_i32()?;
        let origin = cursor.read_i32()?;

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

        Ok(if descriptor_version == EXPECTED_DESCRIPTOR_VERSION {
            cursor.read_4()?;

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

#[derive(Debug, PartialEq, Fail)]
pub enum ImageResourcesDescriptorError {
    #[fail(display = r#"Invalid TypeOS field."#)]
    InvalidTypeOS {},
    #[fail(display = r#"Invalid unit name."#)]
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

    fn read_fields(
        cursor: &mut PsdCursor,
        sub_list: bool,
    ) -> Result<HashMap<String, DescriptorField>, Error> {
        let count = cursor.read_u32()?;
        let mut m = HashMap::with_capacity(count as usize);

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

        for n in 0..count {
            let field = DescriptorStructure::read_descriptor_field(cursor)?;
            vec.push(field);
        }

        Ok(vec)
    }

    fn read_descriptor_field(cursor: &mut PsdCursor) -> Result<DescriptorField, Error> {
        let mut os_type = [0; 4];
        os_type.copy_from_slice(cursor.read_4()?);

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
            OS_TYPE_DOUBLE => DescriptorField::Double(cursor.read_f64()?),
            OS_TYPE_UNIT_FLOAT => {
                DescriptorField::UnitFloat(DescriptorStructure::read_unit_float(cursor)?)
            }
            OS_TYPE_TEXT => DescriptorField::String(cursor.read_unicode_string_padding(1)?),
            OS_TYPE_ENUMERATED => DescriptorField::EnumeratedDescriptor(
                DescriptorStructure::read_enumerated_descriptor(cursor)?,
            ),
            OS_TYPE_LARGE_INTEGER => DescriptorField::LargeInteger(cursor.read_i64()?),
            OS_TYPE_INTEGER => DescriptorField::Integer(cursor.read_i32()?),
            OS_TYPE_BOOL => DescriptorField::Boolean(cursor.read_u8()? > 0),
            OS_TYPE_GLOBAL_OBJECT => {
                DescriptorField::Descriptor(DescriptorStructure::read_descriptor_structure(cursor)?)
            }
            OS_TYPE_CLASS => {
                DescriptorField::Class(DescriptorStructure::read_class_structure(cursor)?)
            }
            OS_TYPE_CLASS2 => {
                DescriptorField::Class(DescriptorStructure::read_class_structure(cursor)?)
            }
            OS_TYPE_ALIAS => {
                DescriptorField::Alias(DescriptorStructure::read_alias_structure(cursor)?)
            }
            OS_TYPE_RAW_DATA => {
                DescriptorField::RawData(DescriptorStructure::read_raw_data(cursor)?)
            }
            _ => return Err(ImageResourcesDescriptorError::InvalidTypeOS {}.into()),
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
    fn read_reference_structure(cursor: &mut PsdCursor) -> Result<Vec<DescriptorField>, Error> {
        let count = cursor.read_u32()?;
        let mut vec = Vec::with_capacity(count as usize);

        for n in 0..count {
            DescriptorStructure::read_key_length(cursor)?;

            let mut os_type = [0; 4];
            os_type.copy_from_slice(cursor.read_4()?);
            vec.push(match &os_type {
                OS_TYPE_PROPERTY => {
                    DescriptorField::Property(DescriptorStructure::read_property_structure(cursor)?)
                }
                OS_TYPE_CLASS3 => {
                    DescriptorField::Class(DescriptorStructure::read_class_structure(cursor)?)
                }
                OS_TYPE_ENUMERATED_REFERENCE => DescriptorField::EnumeratedReference(
                    DescriptorStructure::read_enumerated_reference(cursor)?,
                ),
                OS_TYPE_OFFSET => {
                    DescriptorField::Offset(DescriptorStructure::read_offset_structure(cursor)?)
                }
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
            _ => return Err(ImageResourcesDescriptorError::InvalidUnitName {}.into()),
        })
    }

    fn read_class_structure(cursor: &mut PsdCursor) -> Result<ClassStructure, Error> {
        let name = cursor.read_unicode_string()?;
        let class_id = DescriptorStructure::read_key_length(cursor)?.to_vec();

        Ok(ClassStructure { name, class_id })
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

        Ok(AliasStructure { data })
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
        let length = if length > 0 { length } else { 4 };

        let key = cursor.read(length)?;
        Ok(key)
    }
}
