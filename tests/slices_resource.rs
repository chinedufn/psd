use psd::{ImageResource, Psd};
use std::path::PathBuf;

/// Verify that we properly read the name of a slices resources section.
///
/// For a default PNG there is a slices resource section that has the same name of the PSD file.
///
/// So a file with the name "123.psd" would have a slices resource named "123".
///
/// So, by making fixture files with different name lengths we can verify that we properly parse
/// slice group names of different lengths.
///
/// https://github.com/chinedufn/psd/pull/17
/// https://github.com/chinedufn/psd/pull/18
///
/// cargo test --test slices_resource name_of_psd_has_fifteen_letters -- --exact
#[test]
fn name_of_slices_resource_group() {
    let fixtures = ["1.psd", "12.psd", "123.psd", "1234.psd"];

    for fixture in fixtures.iter() {
        let file = fixtures_dir().join(fixture);
        let expected_slices_name = file.file_stem().unwrap().to_str().unwrap();

        let psd = std::fs::read(&file).unwrap();
        let psd = Psd::from_bytes(&psd).unwrap();

        match &psd.resources()[0] {
            ImageResource::Slices(slices) => {
                assert_eq!(slices.name().as_str(), expected_slices_name);
            }
        };
    }
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/slices-resource")
}
