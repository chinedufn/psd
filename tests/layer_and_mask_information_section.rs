use psd::Psd;

const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];

/// cargo test --test layer_and_mask_information_section layer_and_mask_information_section -- --exact
#[test]
fn layer_and_mask_information_section() {
    let psd = include_bytes!("./fixtures/green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);

    let layer = psd.layer_by_name("First Layer").unwrap();

    assert_eq!(&layer.rgba().unwrap()[..], &GREEN_PIXEL);
}

/// cargo test --test layer_and_mask_information_section layer_with_cyrillic_name -- --exact
#[test]
fn layer_with_cyrillic_name() {
    let psd = include_bytes!("fixtures/green-cyrillic-layer-name-1x1.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);
    psd.layer_by_name("привет").unwrap();
}

/// cargo test --test layer_and_mask_information_section layer_with_chinese_name -- --exact
#[test]
fn layer_with_chinese_name() {
    let psd = include_bytes!("fixtures/green-chinese-layer-name-1x1.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);
    psd.layer_by_name("圆角矩形").unwrap();
}

const TOP_LEVEL_ID: u32 = 1;

/// cargo test --test layer_and_mask_information_section one_group_one_layer_inside -- --exact
#[test]
fn one_group_one_layer_inside() {
    let psd = include_bytes!("fixtures/groups/green-1x1-one-group-one-layer-inside.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);
    assert_eq!(psd.groups().len(), 1);

    // Check group
    let group = psd.group_by_name("group").unwrap();
    assert_eq!(group.id(), TOP_LEVEL_ID);

    let layer_parent_id = psd.layers().get(0).unwrap().parent_id().unwrap();

    assert_eq!(layer_parent_id, group.id());
}

/// cargo test --test layer_and_mask_information_section one_group_one_layer_inside_one_outside -- --exact
#[test]
fn one_group_one_layer_inside_one_outside() {
    let psd =
        include_bytes!("fixtures/groups/green-1x1-one-group-one-layer-inside-one-outside.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    // 1 layer outside + 1 layer inside
    assert_eq!(psd.layers().len(), 2);
    assert_eq!(psd.groups().len(), 1);

    // Check layer outside group
    let layer = psd.layer_by_name("Second Layer").unwrap();
    assert!(layer.parent_id().is_none());

    // Check group
    let group = psd.group_by_name("group").unwrap();
    assert_eq!(group.id(), TOP_LEVEL_ID);

    // Check layer inside group
    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(layer.parent_id().unwrap(), group.id());
}

/// cargo test --test layer_and_mask_information_section two_groups_two_layers_inside -- --exact
#[test]
fn two_groups_two_layers_inside() {
    let psd = include_bytes!("fixtures/groups/green-1x1-two-groups-two-layers-inside.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    // 2 group layer
    assert_eq!(psd.groups().len(), 2);

    // Check first group
    let group = psd.group_by_name("group").unwrap();
    assert_eq!(group.id(), TOP_LEVEL_ID);

    // Check layer inside group
    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(layer.parent_id().unwrap(), group.id());

    // Check second group
    let group = psd.group_by_name("group2").unwrap();
    assert_eq!(group.id(), TOP_LEVEL_ID + 1);
}
