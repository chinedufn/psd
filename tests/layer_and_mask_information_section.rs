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
    let psd = include_bytes!("fixtures/green-cyrillic-layer-name-1x1.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);
    psd.layer_by_name("привет").unwrap();
}

#[test]
fn layer_with_chinese_name() {
    let psd = include_bytes!("fixtures/green-chinese-layer-name-1x1.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);
    psd.layer_by_name("圆角矩形").unwrap();
}

#[test]
fn one_group_one_layer_inside() {
    let psd = include_bytes!("fixtures/green-1x1-one-group-one-layer-inside.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);

    // Check layer
    let group = psd.layer_by_name("group").unwrap();
    group.layers().unwrap().layer_by_name("First Layer").unwrap();
}

#[test]
fn one_group_one_layer_inside_one_outside() {
    let psd = include_bytes!("fixtures/green-1x1-one-group-one-layer-inside-one-outside.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    // 1 layer outside + 1 group layer
    assert_eq!(psd.layers().len(), 2);

    // Check layer outside group
    psd.layer_by_name("Second Layer").unwrap();
    // Check layer inside group
    let group = psd.layer_by_name("group").unwrap();
    group.layers().unwrap().layer_by_name("First Layer").unwrap();
}

#[test]
fn two_groups_two_layers_inside() {
    let psd = include_bytes!("fixtures/green-1x1-two-groups-two-layers-inside.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    // 2 group layer
    assert_eq!(psd.layers().len(), 2);

    // Check first group
    let group = psd.layer_by_name("group").unwrap();
    group.layers().unwrap().layer_by_name("First Layer").unwrap();
    // Check second group
    let group = psd.layer_by_name("group2").unwrap();
    group.layers().unwrap().layer_by_name("Second Layer").unwrap();
}