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
/// cargo test --test image_resources_section image_pascal_string -- --exact
#[test]
fn image_pascal_string() {
    let psd = include_bytes!("./fixtures/non-utf8-no-layer-and-mask-info.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert!(psd.layers().is_empty());
}
