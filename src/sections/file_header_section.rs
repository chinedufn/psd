use crate::sections::PsdCursor;
use failure::{Error, Fail};

/// Bytes representing the string "8BPS".
const EXPECTED_PSD_SIGNATURE: [u8; 4] = [56, 66, 80, 83];
/// Bytes representing the number 1
const EXPECTED_VERSION: [u8; 2] = [0, 1];
/// Bytes representing the Reserved section of the header
const EXPECTED_RESERVED: [u8; 6] = [0; 6];

/// The FileHeaderSection comes from the first 26 bytes in the PSD file.
///
/// We don't store information that isn't useful.
///
/// For example, after validating the PSD signature we won't store it since it is always the
/// same value.
///
/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// The file header contains the basic properties of the image.
///
///
/// | Length | Description                                                                                                                                          |
/// |--------|------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | 4      | Signature: always equal to '8BPS' . Do not try to read the file if the signature does not match this value.                                          |
/// | 2      | Version: always equal to 1. Do not try to read the file if the version does not match this value.<br> (**PSB** version is 2.)                            |
/// | 6      | Reserved: must be zero.                                                                                                                              |
/// | 2      | The number of channels in the image, including any alpha channels. Supported range is 1 to 56.                                                       |
/// | 4      | The height of the image in pixels. Supported range is 1 to 30,000.<br> (**PSB** max of 300,000.)                                                     |
/// | 4      | The width of the image in pixels. Supported range is 1 to 30,000.<br> (**PSB** max of 300,000)                                                       |
/// | 2      | Depth: the number of bits per channel. Supported values are 1, 8, 16 and 32.                                                                         |
/// | 2      | The color mode of the file. Supported values are: Bitmap = 0; Grayscale = 1; Indexed = 2; RGB = 3; CMYK = 4; Multichannel = 7; Duotone = 8; Lab = 9. |
#[derive(Debug)]
pub struct FileHeaderSection {
    pub(in crate) version: PsdVersion,
    pub(in crate) channel_count: ChannelCount,
    pub(in crate) width: PsdWidth,
    pub(in crate) height: PsdHeight,
    pub(in crate) depth: PsdDepth,
    pub(in crate) color_mode: ColorMode,
}

/// Represents an malformed file section header
#[derive(Debug, Fail)]
pub enum FileHeaderSectionError {
    #[fail(
        display = "A file section header is comprised of 26 bytes, you provided {} bytes.",
        length
    )]
    IncorrectLength { length: usize },
    #[fail(
        display = r#"The first four bytes (indices 0-3) of a PSD must always equal [56, 66, 80, 83],
         which in string form is '8BPS'."#
    )]
    InvalidSignature {},
    #[fail(
        display = r#"Bytes 5 and 6 (indices 4-5) must always be [0, 1], Representing a PSD version of 1."#
    )]
    InvalidVersion {},
    #[fail(display = r#"Bytes 7-12 (indices 6-11) must be zeroes"#)]
    InvalidReserved {},
}

impl FileHeaderSection {
    /// Create a FileSectionHeader from the first 26 bytes of a PSD
    ///
    /// TODO: Accept a ColorModeSection along with the bytes so that we can add
    /// any ColorModeSection data to the ColorMode if necessary. Rename this method
    /// to "new" in the process.
    pub fn from_bytes(bytes: &[u8]) -> Result<FileHeaderSection, Error> {
        let mut cursor = PsdCursor::new(bytes);

        // File header section must be 26 bytes long
        if bytes.len() != 26 {
            return Err(FileHeaderSectionError::IncorrectLength {
                length: bytes.len(),
            })?;
        }

        // First four bytes must be '8BPS'
        let signature = cursor.read_4()?;
        if signature != &EXPECTED_PSD_SIGNATURE {
            return Err(FileHeaderSectionError::InvalidSignature {})?;
        }

        // The next 2 bytes represent the version
        let version = cursor.read_2()?;
        if version != &EXPECTED_VERSION {
            return Err(FileHeaderSectionError::InvalidVersion {})?;
        }

        // The next 6 bytes are reserved and should always be 0
        let reserved = cursor.read_6()?;
        if reserved != &EXPECTED_RESERVED {
            return Err(FileHeaderSectionError::InvalidReserved {})?;
        }

        // The next 2 bytes represent the channel count
        let channel = cursor.read_u16()?;
        let channel_count = ChannelCount::new(channel as u8)?;

        // 4 bytes for the height
        let height = cursor.read_u32()?;
        let height = PsdHeight::new(height)?;

        // 4 bytes for the width
        let width = cursor.read_u32()?;
        let width = PsdWidth::new(width)?;

        // 2 bytes for depth
        let depth = cursor.read_2()?;
        let depth = PsdDepth::new(depth[1])?;

        // 2 bytes for color mode
        let color_mode = cursor.read_2()?;
        let color_mode = ColorMode::new(color_mode[1])?;

        Ok(FileHeaderSection {
            version: PsdVersion::One,
            channel_count,
            width,
            height,
            depth,
            color_mode,
        })
    }
}

/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// Version: always equal to 1. Do not try to read the file if the version does not match this value. (**PSB** version is 2.)
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
#[derive(Debug)]
pub enum PsdVersion {
    /// Regular PSD (Not a PSB)
    One,
}

/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// The number of channels in the image, including any alpha channels. Supported range is 1 to 56.
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
#[derive(Debug)]
pub struct ChannelCount(u8);

/// Represents an incorrect channel count
#[derive(Debug, Fail)]
pub enum ChannelCountError {
    #[fail(
        display = "Invalid channel count: {}. Must be 1 <= channel count <= 56",
        channel_count
    )]
    OutOfRange { channel_count: u8 },
}

impl ChannelCount {
    /// Create a new ChannelCount
    pub fn new(channel_count: u8) -> Result<ChannelCount, Error> {
        if channel_count < 1 || channel_count > 56 {
            return Err(ChannelCountError::OutOfRange { channel_count })?;
        }

        Ok(ChannelCount(channel_count))
    }

    /// Return the channel count
    pub fn count(&self) -> u8 {
        self.0
    }
}

/// Represents an incorrect channel count
#[derive(Debug, Fail)]
pub enum PsdSizeError {
    #[fail(display = "Invalid width: {}. Must be 1 <= width <= 30,000", width)]
    WidthOutOfRange { width: u32 },
    #[fail(display = "Invalid height: {}. Must be 1 <= height <= 30,000", height)]
    HeightOutOfRange { height: u32 },
}

/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// The height of the image in pixels. Supported range is 1 to 30,000.
/// (**PSB** max of 300,000.)
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
#[derive(Debug)]
pub struct PsdHeight(pub(in crate) u32);

impl PsdHeight {
    /// Create a new PsdHeight
    pub fn new(height: u32) -> Result<PsdHeight, Error> {
        if height < 1 || height > 30000 {
            return Err(PsdSizeError::HeightOutOfRange { height })?;
        }

        Ok(PsdHeight(height))
    }
}

/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// The width of the image in pixels. Supported range is 1 to 30,000.
/// (*PSB** max of 300,000)
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
#[derive(Debug, Clone, Copy)]
pub struct PsdWidth(pub(in crate) u32);

impl PsdWidth {
    /// Create a new PsdWidth
    pub fn new(width: u32) -> Result<PsdWidth, Error> {
        if width < 1 || width > 30000 {
            return Err(PsdSizeError::WidthOutOfRange { width })?;
        }

        Ok(PsdWidth(width))
    }
}

/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// Depth: the number of bits per channel. Supported values are 1, 8, 16 and 32.
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum PsdDepth {
    One = 1,
    Eight = 8,
    Sixteen = 16,
    ThirtyTwo = 32,
}

/// Represents an incorrect PSD depth
#[derive(Debug, Fail)]
pub enum PsdDepthError {
    #[fail(display = "Depth {} is invalid. Must be 1, 8, 16 or 32", depth)]
    InvalidDepth { depth: u8 },
    #[fail(display = r#"Only 8 and 16 bit depths are supported at the moment.
    If you'd like to see 1 and 32 bit depths supported - please open an issue."#)]
    UnsupportedDepth,
}

impl PsdDepth {
    /// Create a new PsdDepth
    pub fn new(depth: u8) -> Result<PsdDepth, Error> {
        match depth {
            1 => Ok(PsdDepth::One),
            8 => Ok(PsdDepth::Eight),
            16 => Ok(PsdDepth::Sixteen),
            32 => Ok(PsdDepth::ThirtyTwo),
            _ => Err(PsdDepthError::InvalidDepth { depth })?,
        }
    }
}

/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// The color mode of the file. Supported values are: Bitmap = 0; Grayscale = 1; Indexed = 2; RGB = 3; CMYK = 4; Multichannel = 7; Duotone = 8; Lab = 9.
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum ColorMode {
    Bitmap = 0,
    Grayscale = 1,
    // TODO: Indexed(Vec<u8>)
    // Where the vector is the data from the color mode data section
    Indexed = 2,
    Rgb = 3,
    Cmyk = 4,
    Multichannel = 7,
    // TODO: DuoTone(Vec<u8>)
    // Where the vector is the data from the color mode data section.
    Duotone = 8,
    Lab = 9,
}

/// Represents an incorrect color mode
#[derive(Debug, Fail)]
pub enum ColorModeError {
    #[fail(
        display = "Invalid color mode {}. Must be 0, 1, 2, 3, 4, 7, 8 or 9",
        color_mode
    )]
    InvalidColorMode { color_mode: u8 },
}

impl ColorMode {
    /// Create a new ColorMode
    pub fn new(color_mode: u8) -> Result<ColorMode, Error> {
        match color_mode {
            0 => Ok(ColorMode::Bitmap),
            1 => Ok(ColorMode::Grayscale),
            2 => Ok(ColorMode::Indexed),
            3 => Ok(ColorMode::Rgb),
            4 => Ok(ColorMode::Cmyk),
            7 => Ok(ColorMode::Multichannel),
            8 => Ok(ColorMode::Duotone),
            9 => Ok(ColorMode::Lab),
            _ => Err(ColorModeError::InvalidColorMode { color_mode })?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid:
    //   >= 1, <= 56
    #[test]
    fn valid_channel_count() {
        for channel_count in 1..=56 {
            assert!(ChannelCount::new(channel_count).is_ok());
        }
    }

    // Invalid:
    //   < 1, > 56
    #[test]
    fn invalid_channel_count() {
        assert!(ChannelCount::new(0).is_err());
        assert!(ChannelCount::new(57).is_err());
    }

    // We're passing in 25 bytes even though we're supposed to pass in 26 bytes
    #[test]
    fn incorrect_file_header_section_length() {
        let too_short = [0; 25];

        match error_from_bytes(&too_short) {
            FileHeaderSectionError::IncorrectLength { length } => assert_eq!(length, 25),
            _ => panic!("Should have returned incorrect length error"),
        };
    }

    #[test]
    fn first_four_bytes_incorrect() {
        let bytes = make_bytes();

        let error = error_from_bytes(&bytes);

        match error {
            FileHeaderSectionError::InvalidSignature {} => {}
            _ => panic!("Should have returned invalid signature error"),
        };
    }

    #[test]
    fn version_incorrect() {
        let mut bytes = make_bytes();
        bytes[0..4].copy_from_slice(&EXPECTED_PSD_SIGNATURE);

        match error_from_bytes(&bytes) {
            FileHeaderSectionError::InvalidVersion {} => {}
            _ => panic!("Should have returned invalid version error"),
        };
    }

    #[test]
    fn invalid_reserved_section() {
        let mut bytes = make_bytes();
        bytes[0..4].copy_from_slice(&EXPECTED_PSD_SIGNATURE);
        bytes[4..6].copy_from_slice(&EXPECTED_VERSION);

        match error_from_bytes(&bytes) {
            FileHeaderSectionError::InvalidReserved {} => {}
            _ => panic!("Should have returned reserved section error"),
        };
    }

    fn error_from_bytes(bytes: &[u8]) -> FileHeaderSectionError {
        let error = FileHeaderSection::from_bytes(&bytes);
        downcast_file_section_header_error(error)
    }

    fn downcast_file_section_header_error(
        error: Result<FileHeaderSection, Error>,
    ) -> FileHeaderSectionError {
        error
            .err()
            .unwrap()
            .downcast::<FileHeaderSectionError>()
            .unwrap()
    }

    // [0, 1, 2, ..., 25]
    fn make_bytes() -> [u8; 26] {
        let mut bytes = [0; 26];
        for i in 0..26 {
            bytes[i] = i as u8;
        }

        bytes
    }
}
