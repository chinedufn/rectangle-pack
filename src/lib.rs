#![deny(missing_docs)]

use crate::bin_section::{BinSection, MoreSuitableContainersFn};
use crate::layered_rect_groups::{Group, LayeredRectGroups};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Once;
use std::ops::Range;

pub use crate::bin_section::contains_smallest_box;

mod bin_section;
mod layered_rect_groups;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Ord, PartialOrd)]
#[allow(missing_docs)]
pub struct WidthHeightDepth {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

/// Incoming boxes are places into the smallest hole that will fit them.
///
/// "small" vs. "large" is based on the heuristic function.
///
/// A larger heuristic means that the box is larger.
pub type HeuristicFn = dyn Fn(WidthHeightDepth) -> u128;

fn pack_rects<
    InboundId: Debug + Hash + PartialEq + Eq + Clone,
    BinId: Debug + Hash + PartialEq + Eq + Clone,
    GroupId: Debug + Hash + PartialEq + Eq + Clone,
>(
    incoming_groups: &LayeredRectGroups<InboundId, GroupId>,
    mut target_bins: HashMap<BinId, TargetBin>,
    box_size_heuristic: &HeuristicFn,
    more_suitable_containers_fn: &MoreSuitableContainersFn,
) -> Result<RectanglePackOk<InboundId, BinId>, RectanglePackError> {
    let mut packed_locations = HashMap::new();
    let mut bin_stats = HashMap::new();

    'group: for (group_id, incomings) in incoming_groups.group_id_to_inbound_ids.iter() {
        'bin: for (bin_id, bin) in target_bins.iter_mut() {
            let bin_clone = bin.clone();

            'section: for remaining_section in bin_clone.remaining_sections.iter() {
                'incoming: for incoming_id in incomings.iter() {
                    let incoming = incoming_groups.rects[&incoming_id];

                    let placement = remaining_section.try_place(
                        &incoming,
                        more_suitable_containers_fn,
                        box_size_heuristic,
                    );

                    if placement.is_err() {
                        continue 'section;
                    }

                    let (placement, new_sections) = placement.unwrap();

                    unimplemented!()
                }
            }
        }

        return Err(RectanglePackError::NotEnoughBinSpace);
    }

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

fn volume_heuristic(whd: WidthHeightDepth) -> u128 {
    (whd.width * whd.height * whd.depth) as _
}

#[derive(Debug, PartialEq)]
struct RectanglePackOk<InboundId: PartialEq + Eq + Hash, BinId: PartialEq + Eq + Hash> {
    packed_locations: HashMap<InboundId, (BinId, PackedLocation)>,
    bin_stats: HashMap<BinId, BinStats>,
}

#[derive(Debug, PartialEq)]
struct BinStats {
    width: u32,
    height: u32,
    percent_occupied: f32,
}

#[derive(Debug, PartialEq)]
pub struct PackedLocation {
    x: u32,
    y: u32,
    z: u32,
    whd: WidthHeightDepth,
    x_axis_rotation: RotatedBy,
    y_axis_rotation: RotatedBy,
    z_axis_rotation: RotatedBy,
}

#[derive(Debug, PartialEq)]
enum RotatedBy {
    ZeroDegrees,
    NinetyDegrees,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LayeredRect {
    width: u32,
    height: u32,
    depth: u32,
    allow_global_x_axis_rotation: bool,
    allow_global_y_axis_rotation: bool,
    allow_global_z_axis_rotation: bool,
}

impl Into<WidthHeightDepth> for LayeredRect {
    fn into(self) -> WidthHeightDepth {
        WidthHeightDepth {
            width: self.width(),
            height: self.height(),
            depth: self.depth(),
        }
    }
}

impl LayeredRect {
    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        LayeredRect {
            width,
            height,
            depth,
            // Rotation is not yet supported
            allow_global_x_axis_rotation: false,
            allow_global_y_axis_rotation: false,
            allow_global_z_axis_rotation: false,
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

    fn depth(&self) -> u32 {
        self.depth
    }
}

/// An error while attempting to pack rectangles into bins.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum RectanglePackError {
    /// The rectangles can't be placed into the bins. More bin space needs to be provided.
    #[error(r#"Not enough space to place all of the rectangles."#)]
    NotEnoughBinSpace,
}

#[derive(Debug, Clone)]
struct TargetBin {
    max_width: u32,
    max_height: u32,
    max_depth: u32,
    remaining_sections: Vec<BinSection>,
}

impl TargetBin {
    pub fn new(max_width: u32, max_height: u32, max_depth: u32) -> Self {
        let remaining_sections = vec![BinSection::new(
            0,
            0,
            0,
            WidthHeightDepth {
                width: max_width,
                height: max_height,
                depth: max_depth,
            },
        )];

        TargetBin {
            max_width,
            max_height,
            max_depth,
            remaining_sections,
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

        let mut groups: LayeredRectGroups<_, ()> = LayeredRectGroups::new();
        groups.push_rect(InboundId::One, None, LayeredRect::new(3, 1, 1));

        match pack_rects(&groups, targets, &volume_heuristic, &contains_smallest_box).unwrap_err() {
            RectanglePackError::NotEnoughBinSpace => {}
        };
    }

    /// If a multiple rectangles are in a batch then they must be placed together. If there is no
    /// atlas that has enough space to fit them - we return an error.
    #[test]
    fn error_if_a_group_of_rectangles_could_not_fit_into_any_bin() {
        unimplemented!();

        // let mut groups = LayeredRectGroups::new();
        // groups.push_rect(
        //     InboundId::One,
        //     Some(vec![GroupId::Five]),
        //     LayeredRect::new(10, 10, 1),
        // );
        // groups.push_rect(
        //     InboundId::Two,
        //     Some(vec![GroupId::Five]),
        //     LayeredRect::new(10, 10, 1),
        // );
        //
        // let mut targets = HashMap::new();
        // targets.insert(BinId::Three, TargetBin::new(19, 19, 1));
        //
        // match pack_rects(&groups, targets, &volume_heuristic).unwrap_err() {
        //     RectanglePackError::NotEnoughBinSpace {
        //         unplaced_individuals,
        //         unplaced_groups,
        //         ..
        //     } => {
        //         assert_eq!(unplaced_individuals, vec![InboundId::One, InboundId::Two]);
        //         assert_eq!(unplaced_groups, vec![GroupId::Five]);
        //     }
        // };
    }

    /// If we provide a single inbound rectangle and a single bin - it should be placed into that
    /// bin.
    #[test]
    fn one_inbound_rect_one_bin() {
        unimplemented!();
        // let mut groups: LayeredRectGroups<_, ()> = LayeredRectGroups::new();
        // groups.push_rect(InboundId::One, None, LayeredRect::new(1, 2, 1));
        //
        // let mut targets = HashMap::new();
        // targets.insert(BinId::Three, TargetBin::new(5, 5, 1));
        //
        // let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        // let locations = packed.packed_locations;
        //
        // assert_eq!(locations.len(), 1);
        //
        // assert_eq!(locations[&InboundId::One].0, BinId::Three,);
        // assert_eq!(
        //     locations[&InboundId::One].1,
        //     PackedLocation {
        //         left_top_front: [0, 1, 0],
        //         x_axis_rotation: RotatedBy::ZeroDegrees,
        //         y_axis_rotation: RotatedBy::ZeroDegrees,
        //         z_axis_rotation: RotatedBy::ZeroDegrees,
        //     }
        // )
    }

    /// If we have one inbound rect and two bins, it should be placed into the smallest bin.
    #[test]
    fn one_inbound_rect_two_bins() {
        unimplemented!()
        // let mut groups: LayeredRectGroups<_, ()> = LayeredRectGroups::new();
        // groups.push_rect(InboundId::One, None, LayeredRect::new(2, 2, 1));
        //
        // let mut targets = HashMap::new();
        // targets.insert(BinId::Three, TargetBin::new(5, 5, 1));
        // targets.insert(BinId::Four, TargetBin::new(5, 5, 2));
        //
        // let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        // let locations = packed.packed_locations;
        //
        // assert_eq!(locations[&InboundId::One].0, BinId::Four,);
        //
        // assert_eq!(locations.len(), 1);
        // assert_eq!(
        //     locations[&InboundId::One],
        //     PackedLocation {
        //         left_top_front: [0, 1, 0],
        //         right_bottom_back: [1, 0, 0],
        //         x_axis_rotation: RotatedBy::ZeroDegrees,
        //         y_axis_rotation: RotatedBy::ZeroDegrees,
        //         z_axis_rotation: RotatedBy::ZeroDegrees,
        //     }
        // )
    }

    /// If we have two inbound rects and one bin they should both be placed in that bin.
    #[test]
    fn two_inbound_rects_one_bin() {
        unimplemented!()
        // let mut groups: LayeredRectGroups<_, ()> = LayeredRectGroups::new();
        // groups.push_rect(InboundId::One, None, LayeredRect::new(10, 10, 1));
        // groups.push_rect(InboundId::Two, None, LayeredRect::new(10, 10, 1));
        //
        // let mut targets = HashMap::new();
        // targets.insert(BinId::Three, TargetBin::new(20, 20, 2));
        //
        // let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        // let locations = packed.packed_locations;
        //
        // assert_eq!(locations.len(), 2);
        // assert_eq!(
        //     locations[&InboundId::One],
        //     PackedLocation {
        //         bin_id: BinId::Three,
        //         left_top_front: [0, 9],
        //         right_bottom_back: [9, 0],
        //         x_axis_rotation: RotatedBy::ZeroDegrees,
        //         y_axis_rotation: RotatedBy::ZeroDegrees,
        //         z_axis_rotation: RotatedBy::ZeroDegrees,
        //     }
        // );
        // assert_eq!(
        //     locations[&InboundId::Two],
        //     PackedLocation {
        //         bin_id: BinId::Three,
        //         left_top_front: [0, 10],
        //         right_bottom_back: [2, 0],
        //         x_axis_rotation: RotatedBy::ZeroDegrees,
        //         y_axis_rotation: RotatedBy::ZeroDegrees,
        //         z_axis_rotation: RotatedBy::ZeroDegrees,
        //     }
        // )
    }

    /// We have two rectangles and two bins. Each bin has enough space to fit one rectangle.
    ///
    /// 1. First place the largest rectangle into the smallest bin.
    ///
    /// 2. Second place largest rectangle into the next available bin (i.e. the largest one).
    #[test]
    fn two_rects_two_bins() {
        unimplemented!()
        // let mut groups: LayeredRectGroups<_, ()> = LayeredRectGroups::new();
        // groups.push_rect(InboundId::One, None, LayeredRect::new(15, 15, 1));
        // groups.push_rect(InboundId::Two, None, LayeredRect::new(20, 20, 1));
        //
        // let mut targets = HashMap::new();
        // targets.insert(BinId::Three, TargetBin::new(20, 20, 1));
        // targets.insert(BinId::Four, TargetBin::new(50, 50, 1));
        //
        // let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        // let locations = packed.packed_locations;
        //
        // assert_eq!(locations.len(), 2);
        // assert_eq!(
        //     locations[&InboundId::One],
        //     PackedLocation {
        //         bin_id: BinId::Four,
        //         left_top_front: [0, 14],
        //         right_bottom_back: [14, 0],
        //         x_axis_rotation: RotatedBy::ZeroDegrees,
        //         y_axis_rotation: RotatedBy::ZeroDegrees,
        //         z_axis_rotation: RotatedBy::ZeroDegrees,
        //     }
        // );
        // assert_eq!(
        //     locations[&InboundId::Two],
        //     PackedLocation {
        //         bin_id: BinId::Three,
        //         left_top_front: [0, 19],
        //         right_bottom_back: [19, 0],
        //         x_axis_rotation: RotatedBy::ZeroDegrees,
        //         y_axis_rotation: RotatedBy::ZeroDegrees,
        //         z_axis_rotation: RotatedBy::ZeroDegrees,
        //     }
        // )
    }

    /// If there are two sections available to fill - the smaller one should be filled first
    /// (if possible).
    ///
    /// We test this by creating two incoming rectangles. One created two sections - then the
    /// second should get placed into the smaller of the two sections.
    ///
    /// ```text
    /// ┌──────────────┬──▲───────────────┐
    /// │ Second Rect  │  │               │
    /// ├──────────────┴──┤               │
    /// │                 │               │
    /// │  First Placed   │               │
    /// │    Rectangle    │               │
    /// │                 │               │
    /// └─────────────────┴───────────────┘
    /// ```
    #[test]
    fn fills_small_sections_before_large_ones() {
        unimplemented!()
        // let mut targets = HashMap::new();
        // targets.insert(BinId::Three, TargetBin::new(100, 100, 1));
        //
        // let mut groups: LayeredRectGroups<_, ()> = LayeredRectGroups::new();
        //
        // groups.push_rect(InboundId::One, None, LayeredRect::new(50, 90, 1));
        // groups.push_rect(InboundId::Two, None, LayeredRect::new(1, 1, 1));
        //
        // let packed = pack_rects(&groups, targets, &volume_heuristic).unwrap();
        // let locations = packed.packed_locations;
        //
        // assert_eq!(locations.len(), 2);
        // assert_eq!(
        //     locations[&InboundId::One],
        //     PackedLocation {
        //         bin_id: BinId::Four,
        //         left_top_front: [0, 89],
        //         right_bottom_back: [49, 0],
        //         x_axis_rotation: RotatedBy::ZeroDegrees,
        //         y_axis_rotation: RotatedBy::ZeroDegrees,
        //         z_axis_rotation: RotatedBy::ZeroDegrees,
        //     }
        // );
        // assert_eq!(
        //     locations[&InboundId::Two],
        //     PackedLocation {
        //         bin_id: BinId::Three,
        //         left_top_front: [0, 90],
        //         right_bottom_back: [0, 90],
        //         x_axis_rotation: RotatedBy::ZeroDegrees,
        //         y_axis_rotation: RotatedBy::ZeroDegrees,
        //         z_axis_rotation: RotatedBy::ZeroDegrees,
        //     }
        // );
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
