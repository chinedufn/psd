use psd::Psd;

const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];

#[test]
fn layer_and_mask_information_section() {
    let psd = include_bytes!("./fixtures/green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);

    let layer = psd.layer_by_name("First Layer").unwrap();

    assert_eq!(&layer.rgba().unwrap()[..], &GREEN_PIXEL);
}

#[test]
fn layer_with_cyrillic_name() {
    let psd = include_bytes!("./fixtures/green-cyrillic-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);
    
    let layer = psd.layer_by_name("привет").unwrap();

    assert_eq!(&layer.rgba().unwrap()[..], &GREEN_PIXEL);
}

#[test]
fn layer_with_chinese_name() {
    let psd = include_bytes!("./fixtures/green-chinese-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);

    let layer = psd.layer_by_name("圆角矩形").unwrap();

    assert_eq!(&layer.rgba().unwrap()[..], &GREEN_PIXEL);
}