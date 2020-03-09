//! `rectangle-pack` is a library focused on laying out any number of smaller rectangles
//! (both 2d rectangles and 3d rectangular prisms) inside any number of larger rectangles.

#![deny(missing_docs)]

use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;

pub use crate::bin_section::contains_smallest_box;
use crate::bin_section::{BinSection, MoreSuitableContainersFn};
use crate::grouped_rects_to_place::{Group, GroupedRectsToPlace};
pub use crate::target_bin::TargetBin;
use crate::width_height_depth::WidthHeightDepth;

pub use self::box_size_heuristics::{volume_heuristic, BoxSizeHeuristicFn};
pub use self::rect_to_insert::RectToInsert;

mod bin_section;
mod grouped_rects_to_place;

mod rect_to_insert;
mod target_bin;
mod width_height_depth;

mod box_size_heuristics;

/// Information about successfully packed rectangles.
#[derive(Debug, PartialEq)]
pub struct RectanglePackOk<InboundId: PartialEq + Eq + Hash, BinId: PartialEq + Eq + Hash> {
    packed_locations: HashMap<InboundId, (BinId, PackedLocation)>,
    // TODO: Other information such as information about how the bins were packed
    // (perhaps percentage filled)
}

impl<InboundId: PartialEq + Eq + Hash, BinId: PartialEq + Eq + Hash>
    RectanglePackOk<InboundId, BinId>
{
    /// Indicates where every incoming rectangle was placed
    pub fn packed_locations(&self) -> &HashMap<InboundId, (BinId, PackedLocation)> {
        &self.packed_locations
    }
}

/// An error while attempting to pack rectangles into bins.
#[derive(Debug, PartialEq)]
pub enum RectanglePackError {
    /// The rectangles can't be placed into the bins. More bin space needs to be provided.
    NotEnoughBinSpace,
}

impl Display for RectanglePackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            RectanglePackError::NotEnoughBinSpace => {
                f.write_str("Not enough space to place all of the rectangles.")
            }
        }
    }
}

/// Determine how to fit a set of incoming rectangles (2d or 3d) into a set of target bins.
///
/// ## Algorithm
///
/// The algorithm was originally inspired by [rectpack2D] and then modified to work in 3D.
///
/// [rectpack2D]: https://github.com/TeamHypersomnia/rectpack2D
pub fn pack_rects<
    InboundId: Debug + Hash + PartialEq + Eq + Clone,
    BinId: Debug + Hash + PartialEq + Eq + Clone,
    GroupId: Debug + Hash + PartialEq + Eq + Clone,
>(
    rects_to_place: &GroupedRectsToPlace<InboundId, GroupId>,
    target_bins: HashMap<BinId, TargetBin>,
    box_size_heuristic: &BoxSizeHeuristicFn,
    more_suitable_containers_fn: &MoreSuitableContainersFn,
) -> Result<RectanglePackOk<InboundId, BinId>, RectanglePackError> {
    let mut packed_locations = HashMap::new();

    let mut target_bins: Vec<(BinId, TargetBin)> = target_bins.into_iter().collect();
    sort_bins_smallest_to_largest(&mut target_bins, box_size_heuristic);

    let mut group_id_to_inbound_ids: Vec<(&Group<GroupId, InboundId>, &Vec<InboundId>)> =
        rects_to_place.group_id_to_inbound_ids.iter().collect();
    sort_groups_largest_to_smallest(
        &mut group_id_to_inbound_ids,
        rects_to_place,
        box_size_heuristic,
    );

    'group: for (_group_id, incomings) in group_id_to_inbound_ids {
        'incoming: for incoming_id in incomings.iter() {
            for (bin_id, bin) in target_bins.iter_mut() {
                let mut bin_clone = bin.clone();

                'section: while let Some(remaining_section) = bin_clone.remaining_sections.pop() {
                    let rect_to_place = rects_to_place.rects[&incoming_id];

                    let placement = remaining_section.try_place(
                        &rect_to_place,
                        more_suitable_containers_fn,
                        box_size_heuristic,
                    );

                    if placement.is_err() {
                        continue 'section;
                    }

                    let (placement, mut new_sections) = placement.unwrap();
                    sort_by_size_largest_to_smallest(&mut new_sections, box_size_heuristic);

                    bin.remove_filled_section();
                    bin.add_new_sections(new_sections);

                    packed_locations.insert(incoming_id.clone(), (bin_id.clone(), placement));

                    continue 'incoming;
                }
            }

            return Err(RectanglePackError::NotEnoughBinSpace);
        }
    }

    Ok(RectanglePackOk { packed_locations })
}

fn sort_bins_smallest_to_largest<BinId>(
    bins: &mut Vec<(BinId, TargetBin)>,
    box_size_heuristic: &BoxSizeHeuristicFn,
) where
    BinId: Debug + Hash + PartialEq + Eq + Clone,
{
    bins.sort_unstable_by(|a, b| {
        box_size_heuristic(WidthHeightDepth {
            width: a.1.max_width,
            height: a.1.max_height,
            depth: a.1.max_depth,
        })
        .cmp(&box_size_heuristic(WidthHeightDepth {
            width: b.1.max_width,
            height: b.1.max_height,
            depth: b.1.max_depth,
        }))
    });
}

fn sort_by_size_largest_to_smallest(
    items: &mut [BinSection; 3],
    box_size_heuristic: &BoxSizeHeuristicFn,
) {
    items.sort_unstable_by(|a, b| box_size_heuristic(b.whd).cmp(&box_size_heuristic(a.whd)));
}

fn sort_groups_largest_to_smallest<GroupId, InboundId>(
    group_id_to_inbound_ids: &mut Vec<(&Group<GroupId, InboundId>, &Vec<InboundId>)>,
    incoming_groups: &GroupedRectsToPlace<InboundId, GroupId>,
    box_size_heuristic: &BoxSizeHeuristicFn,
) where
    InboundId: Debug + Hash + PartialEq + Eq + Clone,
    GroupId: Debug + Hash + PartialEq + Eq + Clone,
{
    group_id_to_inbound_ids.sort_unstable_by(|a, b| {
        let a_heuristic =
            a.1.iter()
                .map(|inbound| {
                    let rect = incoming_groups.rects[inbound];
                    box_size_heuristic(rect.whd)
                })
                .sum();

        let b_heuristic: u128 =
            b.1.iter()
                .map(|inbound| {
                    let rect = incoming_groups.rects[inbound];
                    box_size_heuristic(rect.whd)
                })
                .sum();

        b_heuristic.cmp(&a_heuristic)
    });
}

/// Describes how and where an incoming rectangle was packed into the target bins
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{pack_rects, volume_heuristic, RectToInsert, RectanglePackError, TargetBin};

    use super::*;

    /// If the provided rectangles can't fit into the provided bins.
    #[test]
    fn error_if_the_rectangles_cannot_fit_into_target_bins() {
        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(2, 100, 1));

        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();
        groups.push_rect(InboundId::One, None, RectToInsert::new(3, 1, 1));

        match pack_rects(&groups, targets, &volume_heuristic, &contains_smallest_box).unwrap_err() {
            RectanglePackError::NotEnoughBinSpace => {}
        };
    }

    /// If we provide a single inbound rectangle and a single bin - it should be placed into that
    /// bin.
    #[test]
    fn one_inbound_rect_one_bin() {
        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();
        groups.push_rect(InboundId::One, None, RectToInsert::new(1, 2, 1));

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(5, 5, 1));

        let packed =
            pack_rects(&groups, targets, &volume_heuristic, &contains_smallest_box).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 1);

        assert_eq!(locations[&InboundId::One].0, BinId::Three,);
        assert_eq!(
            locations[&InboundId::One].1,
            PackedLocation {
                x: 0,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 1,
                    height: 2,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        )
    }

    /// If we have one inbound rect and two bins, it should be placed into the smallest bin.
    #[test]
    fn one_inbound_rect_two_bins() {
        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();
        groups.push_rect(InboundId::One, None, RectToInsert::new(2, 2, 1));

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(5, 5, 1));
        targets.insert(BinId::Four, TargetBin::new(5, 5, 2));

        let packed =
            pack_rects(&groups, targets, &volume_heuristic, &contains_smallest_box).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations[&InboundId::One].0, BinId::Three,);

        assert_eq!(locations.len(), 1);
        assert_eq!(
            locations[&InboundId::One].1,
            PackedLocation {
                x: 0,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 2,
                    height: 2,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        )
    }

    /// If we have two inbound rects the smallest one should be placed first.
    #[test]
    fn places_largest_rectangles_first() {
        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();
        groups.push_rect(InboundId::One, None, RectToInsert::new(10, 10, 1));
        groups.push_rect(InboundId::Two, None, RectToInsert::new(5, 5, 1));

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(20, 20, 2));

        let packed =
            pack_rects(&groups, targets, &volume_heuristic, &contains_smallest_box).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);

        assert_eq!(locations[&InboundId::One].0, BinId::Three,);
        assert_eq!(locations[&InboundId::Two].0, BinId::Three,);

        assert_eq!(
            locations[&InboundId::One].1,
            PackedLocation {
                x: 0,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 10,
                    height: 10,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
        assert_eq!(
            locations[&InboundId::Two].1,
            PackedLocation {
                x: 10,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 5,
                    height: 5,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
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
        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();
        groups.push_rect(InboundId::One, None, RectToInsert::new(15, 15, 1));
        groups.push_rect(InboundId::Two, None, RectToInsert::new(20, 20, 1));

        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(20, 20, 1));
        targets.insert(BinId::Four, TargetBin::new(50, 50, 1));

        let packed =
            pack_rects(&groups, targets, &volume_heuristic, &contains_smallest_box).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);

        assert_eq!(locations[&InboundId::One].0, BinId::Four,);
        assert_eq!(locations[&InboundId::Two].0, BinId::Three,);

        assert_eq!(
            locations[&InboundId::One].1,
            PackedLocation {
                x: 0,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 15,
                    height: 15,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
        assert_eq!(
            locations[&InboundId::Two].1,
            PackedLocation {
                x: 0,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 20,
                    height: 20,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        )
    }

    /// If there are two sections available to fill - the smaller one should be filled first
    /// (if possible).
    ///
    /// We test this by creating two incoming rectangles.
    ///
    /// The largest one is placed and creates two new sections - after which the second, smaller one
    /// should get placed into the smaller of the two new sections.
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
        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(100, 100, 1));

        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();

        groups.push_rect(InboundId::One, None, RectToInsert::new(50, 90, 1));
        groups.push_rect(InboundId::Two, None, RectToInsert::new(1, 1, 1));

        let packed =
            pack_rects(&groups, targets, &volume_heuristic, &contains_smallest_box).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);

        assert_eq!(locations[&InboundId::One].0, BinId::Three,);
        assert_eq!(locations[&InboundId::Two].0, BinId::Three,);

        assert_eq!(
            locations[&InboundId::One].1,
            PackedLocation {
                x: 0,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 50,
                    height: 90,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
        assert_eq!(
            locations[&InboundId::Two].1,
            PackedLocation {
                x: 0,
                y: 90,
                z: 0,
                whd: WidthHeightDepth {
                    width: 1,
                    height: 1,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
    }

    /// Say we have one bin and three rectangles to place within in.
    ///
    /// The first one gets placed and creates two new splits.
    ///
    /// We then attempt to place the second one into the smallest split. It's too big to fit, so
    /// we place it into the largest split.
    ///
    /// After that we place the third rectangle into the smallest split.
    ///
    /// Here we verify that that actually occurs and that we didn't throw away that smallest split
    /// when the second one couldn't fit in it.
    ///
    /// ```text
    /// ┌──────────────┬──────────────┐
    /// │    Third     │              │
    /// ├──────────────┤              │
    /// │              │              │
    /// │              │              │
    /// │              ├──────────────┤
    /// │   First      │              │
    /// │              │    Second    │
    /// │              │              │
    /// └──────────────┴──────────────┘
    /// ```
    #[test]
    fn saves_bin_sections_for_future_use() {
        let mut targets = HashMap::new();
        targets.insert(BinId::Three, TargetBin::new(100, 100, 1));

        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();

        groups.push_rect(InboundId::One, None, RectToInsert::new(50, 95, 1));
        groups.push_rect(InboundId::Two, None, RectToInsert::new(50, 10, 1));
        groups.push_rect(InboundId::Three, None, RectToInsert::new(20, 3, 1));

        let packed =
            pack_rects(&groups, targets, &volume_heuristic, &contains_smallest_box).unwrap();
        let locations = packed.packed_locations;

        assert_eq!(
            locations[&InboundId::One].1,
            PackedLocation {
                x: 0,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 50,
                    height: 95,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
        assert_eq!(
            locations[&InboundId::Two].1,
            PackedLocation {
                x: 50,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 50,
                    height: 10,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
        assert_eq!(
            locations[&InboundId::Three].1,
            PackedLocation {
                x: 0,
                y: 95,
                z: 0,
                whd: WidthHeightDepth {
                    width: 20,
                    height: 3,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum InboundId {
        One,
        Two,
        Three,
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
