use std::collections::HashMap;
use thiserror::Error;

use crate::sections::{AsUnicodeString, PsdCursor, PsdSerialize};
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
#[derive(Debug, Clone)]
pub struct DescriptorStructure {
    pub name: String,
    pub fields: HashMap<String, DescriptorField>,
    pub class_id: Vec<u8>,
}

/// One of
#[derive(Debug, Clone)]
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
    ///
    Class2(ClassStructure),
    ///
    Class3(ClassStructure),
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct AliasStructure {
    pub data: Vec<u8>,
}

/// +----------+----------------------------------------------------------------------------------------+
/// |  Length  |                                      Description                                       |
/// +----------+----------------------------------------------------------------------------------------+
/// | Variable | Type: 4 bytes (length), followed either by string or (if length is zero) 4-byte typeID |
/// | Variable | Enum: 4 bytes (length), followed either by string or (if length is zero) 4-byte enum   |
/// +----------+----------------------------------------------------------------------------------------+
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

impl PsdSerialize for DescriptorStructure {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        AsUnicodeString(&self.name).write(buffer); // Name from ClassID
        DescriptorKey(&self.class_id).write(buffer); // ClassID
        self.fields.write(buffer);
    }
}

struct AsSizedBytes<'a, T>(&'a T)
where
    T: AsRef<[u8]>;

impl<S> PsdSerialize for AsSizedBytes<'_, S>
where
    S: AsRef<[u8]>,
{
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        let inner = self.0.as_ref();
        buffer.write((inner.len() as u32).to_be_bytes());
        buffer.write(inner);
    }
}

struct DescriptorKey<'a, K>(&'a K);

impl<K> PsdSerialize for DescriptorKey<'_, K>
where
    K: AsRef<[u8]>,
{
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        let inner = self.0.as_ref();
        buffer.write((inner.len() as u32).to_be_bytes());

        if inner.is_empty() {
            buffer.write(0_u32.to_be_bytes());
        } else {
            buffer.write(inner);
        }
    }
}

impl PsdSerialize for HashMap<String, DescriptorField> {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        buffer.write((self.len() as u32).to_be_bytes());

        for (key, field) in self {
            DescriptorKey(key).write(buffer); // Key
            field.write(buffer);
        }
    }
}

impl PsdSerialize for Vec<DescriptorField> {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        buffer.write((self.len() as u32).to_be_bytes());

        for field in self {
            field.write(buffer);
        }
    }
}

impl PsdSerialize for UnitFloatStructure {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        let value = match self {
            Self::Angle(v) => {
                buffer.write(UNIT_FLOAT_ANGLE);
                Some(v)
            }
            Self::Density(v) => {
                buffer.write(UNIT_FLOAT_DENSITY);
                Some(v)
            }
            Self::Distance(v) => {
                buffer.write(UNIT_FLOAT_DISTANCE);
                Some(v)
            }
            Self::None => {
                buffer.write(UNIT_FLOAT_NONE);
                None
            }
            Self::Percent(v) => {
                buffer.write(UNIT_FLOAT_PERCENT);
                Some(v)
            }
            Self::Pixels(v) => {
                buffer.write(UNIT_FLOAT_PIXELS);
                Some(v)
            }
        };

        if let Some(value) = value {
            buffer.write(value.to_be_bytes());
        }
    }
}

impl PsdSerialize for ClassStructure {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        AsUnicodeString(&self.name).write(buffer);
        DescriptorKey(&self.class_id).write(buffer);
    }
}

impl PsdSerialize for EnumeratedReference {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        AsUnicodeString(&self.name).write(buffer);
        DescriptorKey(&self.class_id).write(buffer);
        DescriptorKey(&self.key_id).write(buffer);
        DescriptorKey(&self.enum_field).write(buffer);
    }
}

impl PsdSerialize for OffsetStructure {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        AsUnicodeString(&self.name).write(buffer);
        DescriptorKey(&self.class_id).write(buffer);
        buffer.write(self.offset.to_be_bytes());
    }
}

impl PsdSerialize for AliasStructure {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        AsSizedBytes(&self.data).write(buffer);
    }
}

impl PsdSerialize for EnumeratedDescriptor {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        DescriptorKey(&self.type_field).write(buffer);
        DescriptorKey(&self.enum_field).write(buffer);
    }
}

impl PsdSerialize for PropertyStructure {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        AsUnicodeString(&self.name).write(buffer);
        DescriptorKey(&self.class_id).write(buffer);
        DescriptorKey(&self.key_id).write(buffer);
    }
}
impl PsdSerialize for NameStructure {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        AsUnicodeString(&self.name).write(buffer);
        DescriptorKey(&self.class_id).write(buffer);
        AsUnicodeString(&self.value).write(buffer);
    }
}

impl PsdSerialize for DescriptorField {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        match &self {
            Self::Descriptor(item) => {
                buffer.write(OS_TYPE_DESCRIPTOR);
                item.write(buffer);
            }
            Self::Reference(item) => {
                buffer.write(OS_TYPE_REFERENCE);
                //ReferenceStructure(item).write(buffer);
                item.write(buffer);
            }
            Self::UnitFloat(item) => {
                buffer.write(OS_TYPE_UNIT_FLOAT);
                item.write(buffer);
            }
            Self::Double(item) => {
                buffer.write(OS_TYPE_DOUBLE);
                buffer.write(item.to_be_bytes());
            }
            Self::Class(item) => {
                buffer.write(OS_TYPE_CLASS);
                item.write(buffer);
            }
            Self::Class2(item) => {
                buffer.write(OS_TYPE_CLASS2);
                item.write(buffer);
            }
            Self::Class3(item) => {
                buffer.write(OS_TYPE_CLASS3);
                item.write(buffer);
            }
            Self::String(item) => {
                buffer.write(OS_TYPE_TEXT);
                AsUnicodeString(item).write(buffer);
            }
            Self::EnumeratedReference(item) => {
                buffer.write(OS_TYPE_ENUMERATED_REFERENCE);
                item.write(buffer);
            }
            Self::Offset(item) => {
                buffer.write(OS_TYPE_OFFSET);
                item.write(buffer);
            }
            Self::Boolean(item) => {
                buffer.write(OS_TYPE_BOOL);
                let item = match item {
                    true => 1_u8,
                    false => 0_u8,
                };
                buffer.write(item.to_be_bytes());
            }
            Self::Alias(item) => {
                buffer.write(OS_TYPE_ALIAS);
                item.write(buffer);
            }
            Self::List(item) => {
                buffer.write(OS_TYPE_LIST);
                item.write(buffer);
            }
            Self::LargeInteger(item) => {
                buffer.write(OS_TYPE_LARGE_INTEGER);
                buffer.write(item.to_be_bytes());
            }
            Self::Integer(item) => {
                buffer.write(OS_TYPE_INTEGER);
                buffer.write(item.to_be_bytes());
            }
            Self::EnumeratedDescriptor(item) => {
                buffer.write(OS_TYPE_ENUMERATED);
                item.write(buffer);
            }
            Self::RawData(item) => {
                buffer.write(OS_TYPE_RAW_DATA);
                buffer.write(item);
            }
            Self::Property(item) => {
                buffer.write(OS_TYPE_PROPERTY);
                item.write(buffer);
            }
            Self::Identifier(item) => {
                buffer.write(OS_TYPE_IDENTIFIER);
                buffer.write(item.to_be_bytes())
            }
            Self::Index(item) => {
                buffer.write(OS_TYPE_INDEX);
                buffer.write(item.to_be_bytes())
            }
            Self::Name(item) => {
                buffer.write(OS_TYPE_NAME);
                item.write(buffer);
            }
        }
    }
}

impl DescriptorStructure {
    pub(crate) fn read_descriptor_structure(
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
                DescriptorField::Class2(DescriptorStructure::read_class_structure(cursor))
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
