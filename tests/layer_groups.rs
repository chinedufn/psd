use psd::Psd;
const TOP_LEVEL_ID: u32 = 1;

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
///
/// cargo test --test layer_and_mask_information_section one_group_inside_another -- --exact
#[test]
fn one_group_inside_another() {
    let psd = include_bytes!("fixtures/groups/green-1x1-one-group-inside-another.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(psd.layers().len(), 1);
    // parent group + children group
    assert_eq!(psd.groups().len(), 2);

    // Check group
    let group = psd.group_by_name("group outside").unwrap();
    assert_eq!(group.id(), TOP_LEVEL_ID);

    // Check subgroup
    let children_group = psd.group_by_name("group inside").unwrap();
    assert_eq!(children_group.parent_id().unwrap(), group.id());

    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(children_group.id(), layer.parent_id().unwrap());
}

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
/// cargo test --test layer_and_mask_information_section one_group_with_two_subgroups -- --exact
#[test]
fn one_group_with_two_subgroups() {
    let psd = include_bytes!("fixtures/groups/green-1x1-one-group-with-two-subgroups.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert_eq!(6, psd.layers().len());
    assert_eq!(6, psd.groups().len());

    // Check first top-level group
    let outside_group = psd.group_by_name("outside group").unwrap();
    assert_eq!(outside_group.id(), 1);

    // Check first subgroup
    let children_group = psd.group_by_name("first group inside").unwrap();
    assert_eq!(children_group.parent_id().unwrap(), outside_group.id());

    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(children_group.id(), layer.parent_id().unwrap());

    // Check second subgroup
    let children_group = psd.group_by_name("second group inside").unwrap();
    assert_eq!(children_group.parent_id().unwrap(), outside_group.id());

    // Check `sub sub group`
    let sub_sub_group = psd.group_by_name("sub sub group").unwrap();
    assert_eq!(sub_sub_group.parent_id().unwrap(), children_group.id());

    let layer = psd.layer_by_name("Second Layer").unwrap();
    assert_eq!(sub_sub_group.id(), layer.parent_id().unwrap());

    let layer = psd.layer_by_name("Third Layer").unwrap();
    assert_eq!(children_group.id(), layer.parent_id().unwrap());

    // Check third subgroup
    let children_group = psd.group_by_name("third group inside").unwrap();
    assert_eq!(children_group.parent_id().unwrap(), outside_group.id());

    let layer = psd.layer_by_name("Fourth Layer").unwrap();
    assert_eq!(outside_group.id(), layer.parent_id().unwrap());

    // Check top-level Firth Group
    let layer = psd.layer_by_name("Firth Layer").unwrap();
    assert_eq!(layer.parent_id(), None);

    // Check second top-level group
    let outside_group = psd.group_by_name("outside group 2").unwrap();
    assert_eq!(outside_group.id(), 6);

    let layer = psd.layer_by_name("Sixth Layer").unwrap();
    assert_eq!(layer.parent_id().unwrap(), outside_group.id());
}
