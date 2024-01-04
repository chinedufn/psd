use psd::{Psd, PsdGroup};

const GREEN_PIXEL: [u8; 4] = [0, 255, 0, 255];

/// cargo test --test layer_and_mask_information_section layer_and_mask_information_section -- --exact
#[test]
fn layer_and_mask_information_section() {
    let psd = include_bytes!("./fixtures/green-1x1.psd");

    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);

    let layer = psd.layer_by_name("First Layer").unwrap();

    assert_eq!(&layer.rgba()[..], &GREEN_PIXEL);
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

/// cargo test --test layer_and_mask_information_section layer_unicode_string -- --exact
#[test]
fn layer_unicode_string() {
    let psd = include_bytes!("fixtures/luni.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    let mut layer_names: Vec<&str> = psd.layers().iter().map(|l| l.name()).collect();
    layer_names.sort();
    assert_eq!(&layer_names[..], &["1\u{0}", "2 のコピー\u{0}", "3"]);
}

/// cargo test --test layer_and_mask_information_section layer_with_clipping -- --exact
#[test]
fn layer_with_clipping() {
    let psd = include_bytes!("fixtures/green-clipping-10x10.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 3);
    assert_eq!(
        psd.layer_by_name("Clipping base")
            .unwrap()
            .is_clipping_mask(),
        true
    );
    assert_eq!(
        psd.layer_by_name("First clipped layer")
            .unwrap()
            .is_clipping_mask(),
        false
    );
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
    let group = group_by_name(&psd, "group");
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
    let group = group_by_name(&psd, "group");
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
    let group = group_by_name(&psd, "group");
    assert_eq!(group.id(), TOP_LEVEL_ID);

    // Check layer inside group
    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(layer.parent_id().unwrap(), group.id());

    // Check second group
    let group = group_by_name(&psd, "group2");
    assert_eq!(group.id(), TOP_LEVEL_ID + 1);
}

///
/// group structure
/// +---------------+----------+---------+
/// | name          | group_id | parent  |
/// +---------------+----------+---------+
/// | group inside  | 2        | Some(1) | refers to 'group outside'
/// | group outside | 1        | None    |
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
    let psd = include_bytes!("fixtures/groups/green-1x1-one-group-inside-another.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);
    // parent group + children group
    assert_eq!(psd.groups().len(), 2);

    // Check group
    let group = group_by_name(&psd, "group outside");
    assert_eq!(group.id(), TOP_LEVEL_ID);

    // Check subgroup
    let children_group = group_by_name(&psd, "group inside");
    assert_eq!(children_group.parent_id().unwrap(), group.id());

    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(children_group.id(), layer.parent_id().unwrap());
}

///
/// PSD file structure
/// group: outside group, parent: `None`
/// 	group: first group inside, parent: `outside group`
/// 		layer: First Layer, parent: `first group inside`
///
/// 	group: second group inside, parent: `outside group`
/// 		group: sub sub group, parent: `second group inside`
/// 			layer: Second Layer, parent: `sub sub group`
///
/// 		layer: Third Layer, parent: `second group inside`
///
/// 	group: third group inside, parent: `outside group`
///
/// 	layer: Fourth Layer, parent: `outside group`
/// layer: Firth Layer, parent: `None`
///
/// group: outside group 2, parent: `None`
/// 	layer: Sixth Layer, parent: `outside group 2`
///
#[test]
fn one_group_with_two_subgroups() {
    let psd = include_bytes!("fixtures/groups/green-1x1-one-group-with-two-subgroups.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(6, psd.layers().len());
    assert_eq!(6, psd.groups().len());

    // Check first top-level group
    let outside_group = group_by_name(&psd, "outside group");
    assert_eq!(outside_group.id(), 1);

    // Check first subgroup
    let children_group = group_by_name(&psd, "first group inside");
    assert_eq!(children_group.parent_id().unwrap(), outside_group.id());

    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(children_group.id(), layer.parent_id().unwrap());

    // Check second subgroup
    let children_group = group_by_name(&psd, "second group inside");
    assert_eq!(children_group.parent_id().unwrap(), outside_group.id());

    // Check `sub sub group`
    let sub_sub_group = group_by_name(&psd, "sub sub group");
    assert_eq!(sub_sub_group.parent_id().unwrap(), children_group.id());

    let layer = psd.layer_by_name("Second Layer").unwrap();
    assert_eq!(sub_sub_group.id(), layer.parent_id().unwrap());

    let layer = psd.layer_by_name("Third Layer").unwrap();
    assert_eq!(children_group.id(), layer.parent_id().unwrap());

    // Check third subgroup
    let children_group = group_by_name(&psd, "third group inside");
    assert_eq!(children_group.parent_id().unwrap(), outside_group.id());

    let layer = psd.layer_by_name("Fourth Layer").unwrap();
    assert_eq!(outside_group.id(), layer.parent_id().unwrap());

    // Check top-level Firth Group
    let layer = psd.layer_by_name("Firth Layer").unwrap();
    assert_eq!(layer.parent_id(), None);

    // Check second top-level group
    let outside_group = group_by_name(&psd, "outside group 2");
    assert_eq!(outside_group.id(), 6);

    let layer = psd.layer_by_name("Sixth Layer").unwrap();
    assert_eq!(layer.parent_id().unwrap(), outside_group.id());
}

fn group_by_name<'a>(psd: &'a Psd, name: &str) -> &'a PsdGroup {
    psd.groups()
        .iter()
        .find(|group| group.1.name() == name)
        .unwrap()
        .1
}

/// cargo test --test layer_and_mask_information_section layer_larger_than_psd -- --exact
#[test]
fn layer_larger_than_psd() {
    let psd = include_bytes!("./fixtures/layer-larger.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    let larger_layer = psd.layer_by_name("larger").unwrap();
    larger_layer.rgba();
}

/// cargo test --test layer_and_mask_information_section out_of_bounds_layer -- --exact
#[test]
fn out_of_bounds_layer() {
    let psd = include_bytes!("./fixtures/out-of-bounds-layer.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    let larger_layer = psd.layer_by_name("out-of-bounds").unwrap();
    larger_layer.rgba();
}
