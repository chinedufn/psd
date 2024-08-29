use super::descriptor_structure::{DescriptorStructure, ImageResourcesDescriptorError};

use crate::sections::{PsdCursor, PsdDeserialize, PsdSerialize};

const EXPECTED_DESCRIPTOR_VERSION: u32 = 16;

/// Comes from a slices resource block
#[derive(Debug)]
pub enum SlicesImageResource {
    V6(SlicesImageResourceV6),
    V7_8(SlicesImageResourceV7_8),
}

#[allow(missing_docs)]
impl SlicesImageResource {
    pub fn name(&self) -> &str {
        match &self {
            Self::V6(format) => &format.name,
            Self::V7_8(format) => &format.descriptor.name,
        }
    }

    pub fn descriptors(&self) -> Vec<&DescriptorStructure> {
        match &self {
            Self::V6(format) => format
                .blocks
                .iter()
                .filter_map(|b| b.descriptor.as_ref())
                .collect(),
            Self::V7_8(format) => vec![&format.descriptor],
        }
    }
}

impl PsdDeserialize for SlicesImageResource {
    type Error = ImageResourcesDescriptorError;

    /// Slices Resource Format
    /// Adobe Photoshop 6.0 stores slices information for an image in an image resource block.
    /// Adobe Photoshop 7.0 added a descriptor at the end of the block for the individual slice info.
    /// Adobe Photoshop CS and later changed to version 7 or 8 and uses a Descriptor to defined the Slices data.
    ///
    /// +----------+--------------------------------------------------------------------------------------+
    /// |  Length  |                                     Description                                      |
    /// +----------+--------------------------------------------------------------------------------------+
    /// | 4        | Version                                                                              |
    /// | ...      | Fields vary depending on version                                                     |
    /// +----------+--------------------------------------------------------------------------------------+
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let cursor = PsdCursor::new(bytes);
        let version = cursor.peek_i32();
        let bytes = &cursor.get_ref()[cursor.position() as usize..];

        match version {
            x if x == 6 => Ok(Self::V6(SlicesImageResourceV6::from_bytes(bytes)?)),
            x if x == 7 || x == 8 => Ok(Self::V7_8(SlicesImageResourceV7_8::from_bytes(bytes)?)),
            _ => unimplemented!("Slices resource format {version} is currently not supported"),
        }
    }
}

impl PsdSerialize for SlicesImageResource {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        match self {
            Self::V6(format) => format.write(buffer),
            Self::V7_8(format) => format.write(buffer),
        }
    }
}

#[derive(Debug)]
pub struct SlicesImageResourceV6 {
    name: String,
    blocks: Vec<SlicesResourceBlock>,
}

impl PsdDeserialize for SlicesImageResourceV6 {
    type Error = ImageResourcesDescriptorError;

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
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = PsdCursor::new(bytes);

        let _version = cursor.read_i32();
        let _top = cursor.read_i32();
        let _left = cursor.read_i32();
        let _bottom = cursor.read_i32();
        let _right = cursor.read_i32();

        let group_of_slices_name = cursor.read_unicode_string_padding(1);

        let number_of_slices = cursor.read_u32();

        let mut blocks = Vec::new();

        for _ in 0..number_of_slices {
            blocks.push(SlicesResourceBlock::from_bytes(
                &cursor.get_ref()[cursor.position() as usize..],
            )?)
        }

        Ok(Self {
            name: group_of_slices_name,
            blocks,
        })
    }
}

impl PsdSerialize for SlicesImageResourceV6 {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        buffer.write(6_i32.to_be_bytes()); // Version

        let pad = 0_i32.to_be_bytes();
        // Bounding rectangle
        buffer.write(pad); // top
        buffer.write(pad); // left
        buffer.write(pad); // bottom
        buffer.write(pad); // right

        // Name of group of slices: Unicode string
        buffer.write_unicode_string(&self.name);

        // Number of slices to follow
        buffer.write((self.blocks.len() as u32).to_be_bytes());

        for block in &self.blocks {
            block.write(buffer);
        }
    }
}

#[derive(Debug)]
pub struct SlicesResourceBlock {
    descriptor: Option<DescriptorStructure>,
}

impl PsdDeserialize for SlicesResourceBlock {
    type Error = ImageResourcesDescriptorError;

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
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = PsdCursor::new(bytes);

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

        let descriptor = if descriptor_version == EXPECTED_DESCRIPTOR_VERSION {
            cursor.read_4();

            let descriptor = DescriptorStructure::read_descriptor_structure(&mut cursor)?;
            if descriptor.class_id.as_slice() == [0, 0, 0, 0] {
                cursor.seek(pos);
            }

            Some(descriptor)
        } else {
            None
        };

        Ok(Self { descriptor })
    }
}

impl PsdSerialize for SlicesResourceBlock {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        let pad = 0_i32.to_be_bytes();

        buffer.write(pad); // Slice Id
        buffer.write(pad); // Group Id
        buffer.write(pad); // Origin
                           // Skip Associated Layer Id
        buffer.write_unicode_string(""); // Name
        buffer.write(pad); // Type

        // Positions
        buffer.write(pad); // top
        buffer.write(pad); // left
        buffer.write(pad); // bottom
        buffer.write(pad); // right

        buffer.write_unicode_string(""); // URL
        buffer.write_unicode_string(""); // Target
        buffer.write_unicode_string(""); // Message
        buffer.write_unicode_string(""); // Alt Tag

        buffer.write([0_u8]); // Cell text is HTML (Boolean)
        buffer.write_unicode_string(""); // Cell Text

        buffer.write(pad); // Horizontal alignment
        buffer.write(pad); // Vertical alignment
        buffer.write(pad); // Vertical alignment

        buffer.write([0_u8]); // Alpha color
        buffer.write([0_u8]); // Red
        buffer.write([0_u8]); // Green
        buffer.write([0_u8]); // Blue

        if let Some(descriptor) = &self.descriptor {
            buffer.write(EXPECTED_DESCRIPTOR_VERSION.to_be_bytes()); // Descriptor version

            descriptor.write(buffer); // Descriptor
        }
    }
}

#[derive(Debug)]
pub struct SlicesImageResourceV7_8 {
    descriptor: DescriptorStructure,
}

impl PsdDeserialize for SlicesImageResourceV7_8 {
    type Error = ImageResourcesDescriptorError;

    /// Slices header for version 7 or 8
    ///
    /// +----------+--------------------------------------------------------------------------------------+
    /// |  Length  |                                     Description                                      |
    /// +----------+--------------------------------------------------------------------------------------+
    /// | 4        | Version ( = 7 and 8)                                                                 |
    /// | 4        | Descriptor version ( = 16 for Photoshop 6.0).                                        |
    /// | Variable | Descriptor (see See Descriptor structure)                                            |
    /// +----------+--------------------------------------------------------------------------------------+
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = PsdCursor::new(bytes);

        let _version = cursor.read_i32();
        let descriptor_version = cursor.read_i32();
        if descriptor_version != 16 {
            unimplemented!(
                    "Only the version 16 (descriptors) resource format for slices is currently supported"
                );
        }
        let descriptor = DescriptorStructure::read_descriptor_structure(&mut cursor)?;

        Ok(Self { descriptor })
    }
}

impl PsdSerialize for SlicesImageResourceV7_8 {
    fn write<T>(&self, buffer: &mut crate::sections::PsdBuffer<T>)
    where
        T: std::io::Write + std::io::Seek,
    {
        buffer.write(7_i32.to_be_bytes()); // Version
        buffer.write(EXPECTED_DESCRIPTOR_VERSION.to_be_bytes()); // Descriptor Version

        self.descriptor.write(buffer);
    }
}
