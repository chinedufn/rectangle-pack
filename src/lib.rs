#![deny(missing_docs)]

use crate::bin_split::BinSection;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Once;
use std::ops::Range;

mod bin_split;

fn pack_rects<
    InboundId: Debug + Hash + PartialEq + Eq,
    Inbound: Into<LayeredRect>,
    BinId: Debug + Hash + PartialEq + Eq,
    GroupId: Debug + Hash + PartialEq,
>(
    incoming_groups: &LayeredRectGroups<InboundId, GroupId, Inbound>,
    mut target_bins: HashMap<BinId, TargetBin>,
    heuristic: &dyn Fn(u32, u32, u32) -> u128,
) -> Result<RectanglePackOk<InboundId, BinId>, RectanglePackError<InboundId, GroupId>> {
    let mut packed_locations = HashMap::new();
    let mut bin_stats = HashMap::new();

    // for (inbound_id, inbound) in incoming.iter() {
    //     for (bin_id, bin) in target_bins.iter_mut() {
    //         for bin_section in bin.remaining_sections.iter_mut() {
    //             // TODO: Check if inbound can fit into this bin split - if it can then remove the
    //             // split, place it into the split and create two new splits and push those to
    //             // the end of the remaining splits (smallest last)
    //
    //             // If we can't then move on to the next split
    //         }
    //     }
    //
    //     // If we make it here then no bin was able to fit our inbound rect - return an error
    // }

    Ok(RectanglePackOk {
        packed_locations,
        bin_stats,
    })
}

fn volume_heuristic(width: u32, height: u32, layers: u32) -> u128 {
    (width * height * layers) as _
}

#[derive(Debug, PartialEq)]
struct RectanglePackOk<InboundId: PartialEq + Eq + Hash, BinId: PartialEq + Eq + Hash> {
    packed_locations: HashMap<InboundId, PackedLocation<BinId>>,
    bin_stats: HashMap<BinId, BinStats>,
}

#[derive(Debug, PartialEq)]
struct BinStats {
    width: u32,
    height: u32,
    percent_occupied: f32,
}

#[derive(Debug, PartialEq)]
struct PackedLocation<BinId: PartialEq> {
    bin_id: BinId,
    left_top: [u32; 2],
    right_bottom: [u32; 2],
    layers: Range<u32>,
    // TODO: document the getter
    // x_copy = x
    // x = y
    // y = 1 - x_copy
    is_rotated: bool,
}

#[derive(Debug, Copy, Clone)]
struct LayeredRect {
    width: u32,
    height: u32,
    layers: u32,
    allow_rotation: bool,
}

impl IntoIterator for LayeredRect {
    type Item = LayeredRect;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl LayeredRect {
    /// # Panics
    ///
    /// - Panics if the layer count is 0 since that would mean we'd be attempting to place nothing.
    ///
    /// - Panics if allow_rotation is true. This is not yet supported.
    pub fn new(width: u32, height: u32, layers: u32, allow_rotation: bool) -> Self {
        assert!(layers > 0);

        assert!(!allow_rotation);

        LayeredRect {
            width,
            height,
            layers,
            allow_rotation,
        }
    }
}

impl LayeredRect {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn layers(&self) -> u32 {
        self.layers
    }

    fn allow_rotation(&self) -> bool {
        self.allow_rotation
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
enum RectanglePackError<InboundId: Debug + PartialEq, GroupId: Debug> {
    /// The rectangles can't be placed into the bins. More bin space needs to be provided.
    #[error(
        r#"The rectangles cannot fit into the bins.
Placed invidiuals: {placed_individuals:?}
Unplaced invidiuals: {unplaced_individuals:?}
Placed groups: {placed_groups:?}
Unplaced groups: {unplaced_groups:?}
"#
    )]
    NotEnoughBinSpace {
        placed_individuals: Vec<InboundId>,
        unplaced_individuals: Vec<InboundId>,
        placed_groups: Vec<GroupId>,
        unplaced_groups: Vec<GroupId>,
    },
}

enum ImageId {
    One,
    Two,
}

enum BinId {
    One,
    Two,
}

struct TargetBin {
    max_width: u32,
    max_height: u32,
    layers: u32,
    remaining_sections: Vec<BinSection>,
}

impl TargetBin {
    /// # Panics
    ///
    /// Panics if the layer count is 0 since that would mean we'd be attempting to place rectangles
    /// onto nothing.
    pub fn new(max_width: u32, max_height: u32, layers: u32) -> Self {
        assert!(layers > 0);

        let remaining_splits = vec![BinSection::new(0, 0, max_width, max_height, 0, layers)];

        TargetBin {
            max_width,
            max_height,
            layers,
            remaining_sections: remaining_splits,
        }
    }
}

#[derive(Debug)]
struct LayeredRectGroups<InboundId: Hash, GroupId: Hash, Inbound> {
    inbound_id_to_group_ids: HashMap<InboundId, Vec<GroupId>>,
    group_id_to_inbound_ids: HashMap<GroupId, Vec<InboundId>>,
    inbound: HashMap<InboundId, Inbound>,
}

impl<InboundId: Hash + Clone + Eq, GroupdId: Hash + Clone + Eq, Inbound>
    LayeredRectGroups<InboundId, GroupdId, Inbound>
{
    pub fn new() -> Self {
        Self {
            inbound_id_to_group_ids: Default::default(),
            group_id_to_inbound_ids: Default::default(),
            inbound: Default::default(),
        }
    }

    pub fn push_rect(&mut self, inbound_id: InboundId, group_ids: Vec<GroupdId>, inbound: Inbound) {
        self.inbound_id_to_group_ids
            .insert(inbound_id.clone(), group_ids.clone());

        self.inbound.insert(inbound_id.clone(), inbound);

        for group_id in group_ids {
            match self.group_id_to_inbound_ids.entry(group_id) {
                Entry::Occupied(mut o) => {
                    o.get_mut().push(inbound_id.clone());
                }
                Entry::Vacant(v) => {
                    v.insert(vec![inbound_id.clone()]);
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{pack_rects, volume_heuristic, LayeredRect, RectanglePackError, TargetBin};
    use std::collections::HashMap;

    /// If the provided rectangles can't fit into the provided bins.
    #[test]
    fn error_if_the_rectangles_cannot_fit_into_target_bins() {
        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(2, 100, 1));

        let mut groups: LayeredRectGroups<_, (), _> = LayeredRectGroups::new();
        groups.push_rect(InboundId::One, vec![], LayeredRect::new(3, 1, 1, false));

        match pack_rects(&groups, targets, &volume_heuristic)
            .err()
            .unwrap()
        {
            RectanglePackError::NotEnoughBinSpace {
                unplaced_individuals,
                ..
            } => {
                assert_eq!(unplaced_individuals, vec![InboundId::One]);
            }
        };
    }

    /// If a multiple rectangles are in a batch then they must be placed together. If there is no
    /// atlas that has enough space to fit them - we return an error.
    #[test]
    fn error_if_a_group_of_rectangles_could_not_fit_into_any_bin() {
        let mut groups = LayeredRectGroups::new();
        groups.push_rect(
            InboundId::One,
            vec![GroupId::Five],
            LayeredRect::new(10, 10, 1, false),
        );
        groups.push_rect(
            InboundId::Two,
            vec![GroupId::Five],
            LayeredRect::new(10, 10, 1, false),
        );

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(19, 19, 1));

        match pack_rects(&groups, targets, &volume_heuristic).unwrap_err() {
            RectanglePackError::NotEnoughBinSpace {
                unplaced_individuals,
                unplaced_groups,
                ..
            } => {
                assert_eq!(unplaced_individuals, vec![InboundId::One, InboundId::Two]);
                assert_eq!(unplaced_groups, vec![GroupId::Five]);
            }
        };
    }

    /// If we provide a single inbound rectangle and a single bin - it should be placed into that
    /// bin.
    #[test]
    fn one_inbound_rect_one_bin() {
        let mut groups: LayeredRectGroups<_, (), _> = LayeredRectGroups::new();
        groups.push_rect(InboundId::One, vec![], LayeredRect::new(1, 2, 1, false));

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(5, 5, 1));

        let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 1);
        assert_eq!(
            locations[&InboundId::One],
            PackedLocation {
                bin_id: BinId::Three,
                left_top: [0, 1],
                right_bottom: [0, 0],
                layers: 0..1,
                is_rotated: false
            }
        )
    }

    /// If we have one inbound rect and two bins, it should be placed into the smallest bin.
    #[test]
    fn one_inbound_rect_two_bins() {
        let mut groups: LayeredRectGroups<_, (), _> = LayeredRectGroups::new();
        groups.push_rect(InboundId::One, vec![], LayeredRect::new(2, 2, 1, false));

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(5, 5, 1));
        targets.insert(BinId::Four, TargetBin::new(5, 5, 2));

        let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 1);
        assert_eq!(
            locations[&InboundId::One],
            PackedLocation {
                bin_id: BinId::Four,
                left_top: [0, 1],
                right_bottom: [1, 0],
                layers: 1..2,
                is_rotated: false
            }
        )
    }

    /// If we have two inbound rects and one bin they should both be placed in that bin.
    #[test]
    fn two_inbound_rects_one_bin() {
        let mut groups: LayeredRectGroups<_, (), _> = LayeredRectGroups::new();
        groups.push_rect(InboundId::One, vec![], LayeredRect::new(10, 10, 1, false));
        groups.push_rect(InboundId::Two, vec![], LayeredRect::new(10, 10, 1, false));

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(20, 20, 2));

        let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);
        assert_eq!(
            locations[&InboundId::One],
            PackedLocation {
                bin_id: BinId::Three,
                left_top: [0, 9],
                right_bottom: [9, 0],
                layers: 1..2,
                is_rotated: false
            }
        );
        assert_eq!(
            locations[&InboundId::Two],
            PackedLocation {
                bin_id: BinId::Three,
                left_top: [0, 10],
                right_bottom: [2, 0],
                layers: 1..2,
                is_rotated: false
            }
        )
    }

    /// We have two rectangles and two bins. Each bin has enough space to fit one rectangle.
    ///
    /// 1. First place the largest rectangle into the smallest bin.
    ///
    /// 2. Second place largest rectangle into the next available bin (i.e. the largest one).
    #[test]
    fn two_rects_two_bins() {
        let mut groups: LayeredRectGroups<_, (), _> = LayeredRectGroups::new();
        groups.push_rect(InboundId::One, vec![], LayeredRect::new(15, 15, 1, false));
        groups.push_rect(InboundId::Two, vec![], LayeredRect::new(20, 20, 1, false));

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(20, 20, 1));
        targets.insert(BinId::Four, TargetBin::new(50, 50, 1));

        let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);
        assert_eq!(
            locations[&InboundId::One],
            PackedLocation {
                bin_id: BinId::Four,
                left_top: [0, 14],
                right_bottom: [14, 0],
                layers: 0..1,
                is_rotated: false
            }
        );
        assert_eq!(
            locations[&InboundId::Two],
            PackedLocation {
                bin_id: BinId::Three,
                left_top: [0, 19],
                right_bottom: [19, 0],
                layers: 0..1,
                is_rotated: false
            }
        )
    }

    /// If a texture is in two different groups and both groups are getting placed into the same
    /// atlas, don't place the texture twice.
    #[test]
    fn does_not_place_same_texture_twice_into_same_atlas() {
        let group_ids = vec![GroupId::Five, GroupId::Six];

        let mut groups = LayeredRectGroups::new();
        groups.push_rect(
            InboundId::One,
            group_ids,
            LayeredRect::new(15, 15, 1, false),
        );

        let mut targets = HashMap::new();
        targets.insert(BinId::Four, TargetBin::new(50, 50, 1));

        let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);
        assert_eq!(
            locations[&InboundId::One],
            PackedLocation {
                bin_id: BinId::Four,
                left_top: [0, 14],
                right_bottom: [14, 0],
                layers: 0..1,
                is_rotated: false
            }
        );
    }

    /// If one of the textures in a group is already in the atlas it doesn't get considered when
    /// attempting to place the group within that atlas.
    #[test]
    fn group_fits_if_textures_already_in_atlas() {
        let mut groups = LayeredRectGroups::new();
        groups.push_rect(
            InboundId::One,
            vec![GroupId::Five, GroupId::Six],
            LayeredRect::new(15, 15, 1, false),
        );
        groups.push_rect(
            InboundId::Two,
            vec![GroupId::Six],
            LayeredRect::new(20, 20, 1, false),
        );

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(20, 20, 1));
        targets.insert(BinId::Four, TargetBin::new(50, 50, 1));

        let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);
        assert_eq!(
            locations[&InboundId::One],
            PackedLocation {
                bin_id: BinId::Four,
                left_top: [0, 14],
                right_bottom: [14, 0],
                layers: 0..1,
                is_rotated: false
            }
        );
        assert_eq!(
            locations[&InboundId::Two],
            PackedLocation {
                bin_id: BinId::Three,
                left_top: [0, 19],
                right_bottom: [19, 0],
                layers: 0..1,
                is_rotated: false
            }
        );
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum InboundId {
        One,
        Two,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum BinId {
        Three,
        Four,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum GroupId {
        Five,
        Six,
    }
}
