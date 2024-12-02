use anyhow::Result;
use psd::Psd;

#[test]
fn visibility() -> Result<()> {
    let psd = include_bytes!("./fixtures/visibility.psd");
    let psd = Psd::from_bytes(psd)?;

    let layers = psd.layers();
    layers.iter().for_each(|layer| {
        match layer.name() {
            "visible" => assert!(layer.visible()),
            "invisible" => assert!(!layer.visible()),
            _ => (),
        }
    });
    Ok(())
}