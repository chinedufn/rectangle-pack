#![deny(missing_docs)]

use crate::bin_split::BinSection;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Once;

mod bin_split;

fn pack_rects<
    InboundId: Debug + Hash,
    Inbound: IntoIterator<Item = LayeredRect>,
    BinId: Hash,
    GroupId: Hash + Debug,
>(
    incoming_groups: &LayeredRectGroups<InboundId, GroupId, Inbound>,
    mut target_bins: HashMap<BinId, TargetBin>,
    heuristic: &dyn Fn(u32, u32, u32) -> u128,
) -> Result<RectanglePackOk<InboundId, BinId>, RectanglePackError<InboundId, GroupId>> {
    let mut packed_locations = HashMap::new();

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

    Ok(RectanglePackOk { packed_locations })
}

fn volume_heuristic(width: u32, height: u32, layers: u32) -> u128 {
    (width * height * layers) as _
}

struct RectanglePackOk<InboundId, BinId> {
    packed_locations: HashMap<InboundId, PackedLocation<BinId>>,
}

struct PackedLocation<BinId> {
    bin_id: BinId,
    top_left: [u32; 2],
    bottom_right: [u32; 2],
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
    /// Panics if the layer count is 0 since that would mean we'd be attempting to place nothing.
    pub fn new(width: u32, height: u32, layers: u32, allow_rotation: bool) -> Self {
        assert!(layers > 0);

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

#[derive(Debug, thiserror::Error)]
enum RectanglePackError<InboundId: Debug, GroupId: Debug> {
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

impl<InboundId: Eq + Hash + Clone, GroupdId: Hash + Eq + Clone, Inbound>
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
    fn error_if_a_batch_of_rectangles_could_not_fit_into_an_atlas() {
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

        match pack_rects(&groups, targets, &volume_heuristic)
            .err()
            .unwrap()
        {
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
        unimplemented!()
    }

    /// If we have one inbound rect and two bins, it should be placed into the smallest bin.
    #[test]
    fn one_inbound_rect_two_bins() {
        unimplemented!()
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
