use anyhow::Result;
use psd::Psd;

/// Verify that `Psd::into_bytes` produces a byte stream that can be reparsed and preserves key
/// high-level invariants for a small, known fixture.
#[test]
fn basic_roundtrip() -> Result<()> {
    // Use a tiny fixture to keep the test lightweight and fast
    let bytes = include_bytes!("./fixtures/groups/green-1x1-one-group-one-layer-inside-one-outside.psd");

    let original = Psd::from_bytes(bytes)?;

    // Serialize back to bytes
    let out = original.into_bytes()?;

    // Basic file signature sanity.
    assert_eq!(&out[0..4], &[56, 66, 80, 83], "output should start with 8BPS");

    // Parse again
    let reparsed = Psd::from_bytes(&out)?;

    // Assert a couple of key invariants remain stable across roundtrip
    assert_eq!(original.width(), reparsed.width(), "width should match after roundtrip");
    assert_eq!(original.height(), reparsed.height(), "height should match after roundtrip");
    assert_eq!(original.depth(), reparsed.depth(), "depth should match after roundtrip");
    assert_eq!(
        original.color_mode(),
        reparsed.color_mode(),
        "color mode should match after roundtrip"
    );
    assert_eq!(
        original.compression(),
        reparsed.compression(),
        "compression should match after roundtrip"
    );

    // Layer count should be preserved for supported subset
    assert_eq!(original.layers().len(), reparsed.layers().len(), "layer count should match");

    // Resource list should be preserved.
    assert_eq!(
        original.resources().len(),
        reparsed.resources().len(),
        "resource count should match"
    );

    // Group structure should be preserved.
    assert_eq!(
        original.groups().len(),
        reparsed.groups().len(),
        "group count should match"
    );
    assert_eq!(
        original.group_ids_in_order(),
        reparsed.group_ids_in_order(),
        "group ordering should match"
    );

    // Compare merged RGBA length (content may differ in some edge cases, but size should match)
    assert_eq!(original.rgba().len(), reparsed.rgba().len(), "rgba length should match");

    Ok(())
}
