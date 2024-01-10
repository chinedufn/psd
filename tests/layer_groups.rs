use psd::{NodeAction, NodeType, Psd, PsdGroup};
const TOP_LEVEL_ID: u32 = 1;

/// Verify that we can get a group by it's ID.
#[test]
fn group_by_id() {
    let psd = include_bytes!("fixtures/groups/green-1x1-one-group-inside-another.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    assert!(psd.groups().get(&0).is_none());

    assert_eq!(psd.group_ids_in_order(), &[2, 1]);

    assert_eq!(psd.groups().get(&1).unwrap().name(), "group outside");
    assert_eq!(psd.groups().get(&2).unwrap().name(), "group inside");
}

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
    let group = group_by_name(&psd, "group outside");
    assert_eq!(group.id(), TOP_LEVEL_ID);

    // Check subgroup
    let children_group = group_by_name(&psd, "group inside");
    assert_eq!(children_group.parent_id().unwrap(), group.id());

    let layer = psd.layer_by_name("First Layer").unwrap();
    assert_eq!(children_group.id(), layer.parent_id().unwrap());
}

/// PSD file structure
/// group: outside group, parent: `None`
///     group: first group inside, parent: `outside group`
///         layer: First Layer, parent: `first group inside`
///
///     group: second group inside, parent: `outside group`
///         group: sub sub group, parent: `second group inside`
///             layer: Second Layer, parent: `sub sub group`
///
///         layer: Third Layer, parent: `second group inside`
///
///     group: third group inside, parent: `outside group`
///
///     layer: Fourth Layer, parent: `outside group`
/// layer: Firth Layer, parent: `None`
///
/// group: outside group 2, parent: `None`
///     layer: Sixth Layer, parent: `outside group 2`
///
/// cargo test --test layer_and_mask_information_section one_group_with_two_subgroups -- --exact
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

/// cargo test --test layer_and_mask_information_section tree_one_group_with_two_subgroups -- --exact
#[test]
fn tree_one_group_with_two_subgroups() {
    let psd = include_bytes!("fixtures/groups/green-1x1-one-group-with-two-subgroups.psd");
    let psd = Psd::from_bytes(psd).unwrap();

    let tree = psd.tree();
    let root_node = tree.borrow();

    // "outside group"
    let outside_group_node = root_node.child(0).unwrap();
    let outside_group = match outside_group_node.borrow().content() {
        Some(NodeType::Group(group)) => group,
        _ => panic!("Expected 'outside group' to be a group"),
    };
    assert_eq!(outside_group.id(), 1);

    // "first group inside"
    let first_group_inside_node = outside_group_node.borrow().child(0).unwrap();
    let first_group_inside = match first_group_inside_node.borrow().content() {
        Some(NodeType::Group(group)) => group,
        _ => panic!("Expected 'first group inside' to be a group"),
    };
    assert_eq!(first_group_inside.parent_id().unwrap(), outside_group.id());

    // "First Layer"
    let first_layer_node = first_group_inside_node.borrow().child(0).unwrap();
    let first_layer = match first_layer_node.borrow().content() {
        Some(NodeType::Layer(layer)) => layer,
        _ => panic!("Expected 'First Layer' to be a layer"),
    };
    assert_eq!(first_group_inside.id(), first_layer.parent_id().unwrap());

    // "second group inside"
    let second_group_inside_node = outside_group_node.borrow().child(1).unwrap();
    let second_group_inside = match second_group_inside_node.borrow().content() {
        Some(NodeType::Group(group)) => group,
        _ => panic!("Expected 'second group inside' to be a group"),
    };
    assert_eq!(second_group_inside.parent_id().unwrap(), outside_group.id());

    // "sub sub group"
    let sub_sub_group_node = second_group_inside_node.borrow().child(0).unwrap();
    let sub_sub_group = match sub_sub_group_node.borrow().content() {
        Some(NodeType::Group(group)) => group,
        _ => panic!("Expected 'sub sub group' to be a group"),
    };
    assert_eq!(sub_sub_group.parent_id().unwrap(), second_group_inside.id());

    // "Second Layer"
    let second_layer_node = sub_sub_group_node.borrow().child(0).unwrap();
    let second_layer = match second_layer_node.borrow().content() {
        Some(NodeType::Layer(layer)) => layer,
        _ => panic!("Expected 'Second Layer' to be a layer"),
    };
    assert_eq!(sub_sub_group.id(), second_layer.parent_id().unwrap());

    // "Third Layer"
    let third_layer_node = second_group_inside_node.borrow().child(1).unwrap();
    let third_layer = match third_layer_node.borrow().content() {
        Some(NodeType::Layer(layer)) => layer,
        _ => panic!("Expected 'Third Layer' to be a layer"),
    };
    assert_eq!(second_group_inside.id(), third_layer.parent_id().unwrap());

    // "third group inside"
    let third_group_inside_node = outside_group_node.borrow().child(2).unwrap();
    let third_group_inside = match third_group_inside_node.borrow().content() {
        Some(NodeType::Group(group)) => group,
        _ => panic!("Expected 'third group inside' to be a group"),
    };
    assert_eq!(third_group_inside.parent_id().unwrap(), outside_group.id());

    // "Fourth Layer"
    let fourth_layer_node = outside_group_node.borrow().child(3).unwrap();
    let fourth_layer = match fourth_layer_node.borrow().content() {
        Some(NodeType::Layer(layer)) => layer,
        _ => panic!("Expected 'Fourth Layer' to be a layer"),
    };
    assert_eq!(outside_group.id(), fourth_layer.parent_id().unwrap());

    // "Firth Layer"
    let firth_layer_node = root_node.child(1).unwrap();
    let firth_layer = match firth_layer_node.borrow().content() {
        Some(NodeType::Layer(layer)) => layer,
        _ => panic!("Expected 'Firth Layer' to be a layer"),
    };
    assert!(firth_layer.parent_id().is_none());

    // "outside group 2"
    let outside_group_2_node = root_node.child(2).unwrap();
    let outside_group_2 = match outside_group_2_node.borrow().content() {
        Some(NodeType::Group(group)) => group,
        _ => panic!("Expected 'outside group 2' to be a group"),
    };
    assert_eq!(outside_group_2.id(), 6);

    // "Sixth Layer"
    let sixth_layer_node = outside_group_2_node.borrow().child(0).unwrap();
    let sixth_layer = match sixth_layer_node.borrow().content() {
        Some(NodeType::Layer(layer)) => layer,
        _ => panic!("Expected 'Sixth Layer' to be a layer"),
    };
    assert_eq!(outside_group_2.id(), sixth_layer.parent_id().unwrap());
}

/// Verify that we can properly load an RLEcompressed empty channel (caused by a group from GIMP)
#[test]
fn rle_compressed_empty_channel() {
    let psd = include_bytes!("fixtures/groups/rle-compressed-empty-channel.psd");
    let psd = Psd::from_bytes(psd);
    assert!(psd.is_ok());
}

fn group_by_name<'a>(psd: &'a Psd, name: &str) -> &'a PsdGroup {
    psd.groups()
        .iter()
        .find(|group| group.1.name() == name)
        .unwrap()
        .1
}
