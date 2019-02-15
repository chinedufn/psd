use failure::{Error, Fail};

/// # Adobe Docs
///
/// The file header contains the basic properties of the image.
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
pub struct FileHeaderSection {
    signature: PsdSignature,
    version: PsdVersion,
    channel_count: ChannelCount,
    width: PsdWidth,
    height: PsdHeight,
    depth: PsdDepth,
    color_mode: ColorMode,
}

/// # Adobe Docs
///
/// Signature: always equal to '8BPS'. Do not try to read the file if the signature does not match this value.
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
pub enum PsdSignature {
    EightBPS,
}

/// # Adobe Docs
///
/// Version: always equal to 1. Do not try to read the file if the version does not match this value. (**PSB** version is 2.)
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
pub enum PsdVersion {
    /// Not PSB
    One,
}

/// # Adobe Docs
///
/// The number of channels in the image, including any alpha channels. Supported range is 1 to 56.
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
pub struct ChannelCount(u8);

/// Represents an incorrect channel count
#[derive(Debug, Fail)]
pub enum ChannelCountError {
    #[fail(display = "Invalid channel count: {}. Must be 1 <= channel count <= 56", channel_count)]
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
}

/// Represents an incorrect channel count
#[derive(Debug, Fail)]
pub enum PsdSizeError {
    #[fail(display = "Invalid width: {}. Must be 1 <= width <= 30,000", width)]
    WidthOutOfRange { width: u16 },
    #[fail(display = "Invalid height: {}. Must be 1 <= height <= 30,000", height)]
    HeightOutOfRange { height: u16 },
}

/// # Adobe Docs
///
/// The height of the image in pixels. Supported range is 1 to 30,000.
/// (**PSB** max of 300,000.)
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
pub struct PsdHeight(u16);

impl PsdHeight {
    /// Create a new PsdHeight
    pub fn new(height: u16) -> Result<PsdHeight, Error> {
        if height < 1 || height > 30000 {
            return Err(PsdSizeError::HeightOutOfRange { height })?;
        }

        Ok(PsdHeight(height))
    }
}

impl PsdWidth {
    /// Create a new PsdWidth
    pub fn new(width: u16) -> Result<PsdWidth, Error> {
        if width < 1 || width > 30000 {
            return Err(PsdSizeError::WidthOutOfRange { width })?;
        }

        Ok(PsdWidth(width))
    }
}

/// # Adobe Docs
///
/// The width of the image in pixels. Supported range is 1 to 30,000.
/// (*PSB** max of 300,000)
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
pub struct PsdWidth(u16);

/// # Adobe Docs
///
/// Depth: the number of bits per channel. Supported values are 1, 8, 16 and 32.
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
pub enum PsdDepth {
    One,
    Eight,
    Sixteen,
    ThirtyTwo,
}

/// # Adobe Docs
///
/// The color mode of the file. Supported values are: Bitmap = 0; Grayscale = 1; Indexed = 2; RGB = 3; CMYK = 4; Multichannel = 7; Duotone = 8; Lab = 9.
///
/// via: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/
pub enum ColorMode {
    Bitmap = 0,
    Grayscale = 1,
    Indexed = 2,
    Rgb = 3,
    Cmyk = 4,
    Multichannel = 7,
    Duotone = 8,
    Lab = 9,
}

#[cfg(test)]
mod tests {
    use super::*;

    // >= 1, <= 56
    #[test]
    fn valid_channel_count() {
        for channel_count in 1..=56 {
            assert!(ChannelCount::new(channel_count).is_ok());
        }
    }

    // <= 1, >= 56
    #[test]
    fn invalid_channel_count() {
        assert!(ChannelCount::new(0).is_err());
        assert!(ChannelCount::new(57).is_err());
    }

}
