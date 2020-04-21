use psd::Psd;

/// Test to verify an issue where parsing a default PSD file with a name that had 15 letters
/// panics while parsing the slices resource section.
///
/// Files with names with 23 characters were also crashing - so the issue isn't the number 15
/// in particular.
///
/// cargo test --test slices_resource name_of_psd_has_fifteen_letters -- --exact
#[test]
fn name_of_psd_has_fifteen_letters() {
    let psd = include_bytes!("fixtures/fifteen-letters.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 0);
}
