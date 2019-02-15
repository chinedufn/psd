//! Data structures and methods for working with PSD files.
//!
//! psd spec: https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/

#![deny(missing_docs)]

mod file_header_section;

/// Represents the contents of a PSD file
///
/// ## PSB Support
///
/// We do not currently support PSB since the originally authors didn't need it, but adding
/// support should be trivial. If you'd like to support PSB please open an issue.
pub struct Psd {
}