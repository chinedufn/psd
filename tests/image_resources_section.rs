use psd::{DescriptorField, ImageResource, Psd};

/// In this test we check that root descriptor's `bounds` field is equal to 1
/// So, then fields parsed correctly
///
/// cargo test --test image_resources_section image_check_1x1p_bound_field -- --exact
#[test]
fn image_check_1x1p_bound_field() {
    let psd = include_bytes!("./fixtures/two-layers-red-green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    let descriptors = match &psd.resources()[0] {
        ImageResource::Slices(s) => s.descriptors(),
    };
    let descriptor = descriptors.get(0).unwrap();
    let bounds = descriptor.fields.get("bounds").unwrap();

    if let DescriptorField::Descriptor(d) = bounds {
        match d.fields.get("Rght").unwrap() {
            DescriptorField::Integer(v) => assert_eq!(*v, 1),
            _ => panic!("expected integer"),
        }

        match d.fields.get("Btom").unwrap() {
            DescriptorField::Integer(v) => assert_eq!(*v, 1),
            _ => panic!("expected integer"),
        }
    } else {
        panic!("expected descriptor");
    }
}

/// In this test we check that root descriptor's `bounds` field is equal to 16
/// So, then fields parsed correctly
///
/// cargo test --test image_resources_section image_check_16x16p_bound_field -- --exact
#[test]
fn image_check_16x16p_bound_field() {
    let psd = include_bytes!("./fixtures/16x16-rle-partially-opaque.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    let descriptors = match &psd.resources()[0] {
        ImageResource::Slices(s) => s.descriptors(),
    };
    let descriptor = descriptors.get(0).unwrap();
    let bounds = descriptor.fields.get("bounds").unwrap();

    if let DescriptorField::Descriptor(d) = bounds {
        match d.fields.get("Rght").unwrap() {
            DescriptorField::Integer(v) => assert_eq!(*v, 16),
            _ => panic!("expected integer"),
        }

        match d.fields.get("Btom").unwrap() {
            DescriptorField::Integer(v) => assert_eq!(*v, 16),
            _ => panic!("expected integer"),
        }
    } else {
        panic!("expected descriptor");
    }
}

/// The image contains a non-UTF-8 Pascal string of even length in its image resource block.
///
/// cargo test --test image_resources_section image_non_utf8_pascal_string -- --exact
#[test]
fn image_non_utf8_pascal_string() {
    let psd = include_bytes!("./fixtures/non-utf8-pascal-string.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert!(psd.layers().is_empty());
}

/// The image contains a Pascal string of odd length in its image resource block.
///
/// cargo test --test image_resources_section image_odd_length_pascal_string -- --exact
#[test]
fn image_odd_length_pascal_string() {
    let psd = include_bytes!("./fixtures/odd-length-pascal-string.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert!(psd.layers().is_empty());
}

/// Verify that ICC profile extraction works for PSD files with an embedded profile.
///
/// cargo test --test image_resources_section icc_profile_extracted -- --exact
#[test]
fn icc_profile_extracted() {
    let psd = include_bytes!("./fixtures/green-1x1.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    let icc = psd.icc_profile().expect("expected ICC profile");
    // ICC profiles start with a 4-byte size field, then 4 bytes of padding/reserved,
    // then a 4-byte profile/device class signature. Minimum valid size is 128 bytes (header).
    assert!(icc.len() >= 128, "ICC profile too small: {} bytes", icc.len());
    // The first 4 bytes are the profile size as a big-endian u32
    let profile_size = u32::from_be_bytes([icc[0], icc[1], icc[2], icc[3]]) as usize;
    assert_eq!(profile_size, icc.len(), "ICC profile size field mismatch");
}

/// Verify that PSD files without an ICC profile return None.
///
/// cargo test --test image_resources_section icc_profile_absent -- --exact
#[test]
fn icc_profile_absent() {
    // layer-larger.psd has no ICC profile based on manual inspection
    let psd = include_bytes!("./fixtures/layer-larger.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert!(psd.icc_profile().is_none(), "expected no ICC profile");
}
