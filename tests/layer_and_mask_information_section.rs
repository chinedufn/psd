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
    assert_eq!(psd.groups().len(), 1);

    // Check group
    let group = psd.group_by_name("group").unwrap();
    assert_eq!(group.id(), 0);

    let layer_parent_id = psd.layers().
        get(0).
        unwrap().
        parent_id().
        unwrap();

    assert_eq!(layer_parent_id, group.id());
}

#[test]
fn one_group_one_layer_inside_one_outside() {
    let psd = include_bytes!("fixtures/green-1x1-one-group-one-layer-inside-one-outside.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    // 1 layer outside + 1 layer inside
    assert_eq!(psd.layers().len(), 2);
    assert_eq!(psd.groups().len(), 1);

    // Check layer outside group
    let layer = psd.layer_by_name("Second Layer").unwrap();
    assert!(layer.parent_id().is_none());

    // Check group
    let group = psd.group_by_name("group").unwrap();
    assert_eq!(group.id(), 0);

    // Check layer inside group
    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(layer.parent_id().unwrap(), group.id());
}

#[test]
fn two_groups_two_layers_inside() {
    let psd = include_bytes!("fixtures/green-1x1-two-groups-two-layers-inside.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    // 2 group layer
    assert_eq!(psd.groups().len(), 2);

    // Check first group
    let group = psd.group_by_name("group").unwrap();
    assert_eq!(group.id(), 0);

    // Check layer inside group
    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(layer.parent_id().unwrap(), group.id());

    // Check second group
    let group = psd.group_by_name("group2").unwrap();
    assert_eq!(group.id(), 1);
}

///
/// group structure
/// +---------------+----------+---------+
/// | name          | group_id | parent  |
/// +---------------+----------+---------+
/// | group inside  | 1        | Some(0) | refers to 'group outside'
/// | group outside | 0        | None    |
/// +------------------------------------+
///
/// layer structure
/// +-------------+-----+---------+
/// | name        | idx | parent  |
/// +-------------+-----+---------+
/// | First Layer | 0   | Some(1) |  refers to 'group inside'
/// +-------------+-----+---------+
#[test]
fn one_group_inside_another() {
    let psd = include_bytes!("fixtures/green-1x1-one-group-inside-another.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);
    // parent group + children group
    assert_eq!(psd.groups().len(), 2);

    // Check group
    let group = psd.group_by_name("group outside").unwrap();
    assert_eq!(group.id(), 0);


    // Check subgroup
    let children_group = psd.group_by_name("group inside").unwrap();
    assert_eq!(children_group.parent_id().unwrap(), group.id());

    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(children_group.id(), layer.parent_id().unwrap());
}