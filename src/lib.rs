//! `rectangle-pack` is a library focused on laying out any number of smaller rectangles
//! (both 2d rectangles and 3d rectangular prisms) inside any number of larger rectangles.
#![cfg_attr(not(std), no_std)]
#![deny(missing_docs)]

#[macro_use]
extern crate alloc;

#[cfg(not(std))]
use alloc::collections::BTreeMap as KeyValMap;
#[cfg(std)]
use std::collections::HashMap as KeyValMap;

use alloc::{collections::BTreeMap, vec::Vec};

use core::{
    fmt::{Debug, Display, Error as FmtError, Formatter},
    hash::Hash,
};

pub use crate::bin_section::contains_smallest_box;
pub use crate::bin_section::BinSection;
pub use crate::bin_section::ComparePotentialContainersFn;
use crate::grouped_rects_to_place::Group;
pub use crate::grouped_rects_to_place::GroupedRectsToPlace;
pub use crate::target_bin::TargetBin;
use crate::width_height_depth::WidthHeightDepth;

pub use self::box_size_heuristics::{volume_heuristic, BoxSizeHeuristicFn};
pub use self::rect_to_insert::RectToInsert;
pub use crate::packed_location::PackedLocation;

mod bin_section;
mod grouped_rects_to_place;

mod packed_location;
mod rect_to_insert;
mod target_bin;
mod width_height_depth;

mod box_size_heuristics;

/// Determine how to fit a set of incoming rectangles (2d or 3d) into a set of target bins.
///
/// ## Example
///
/// ```
/// //! A basic example of packing rectangles into target bins
///
/// use rectangle_pack::{
///     GroupedRectsToPlace,
///     RectToInsert,
///     pack_rects,
///     TargetBin,
///     volume_heuristic,
///     contains_smallest_box
/// };
/// use std::collections::BTreeMap;
///
/// // A rectangle ID just needs to meet these trait bounds (ideally also Copy).
/// // So you could use a String, PathBuf, or any other type that meets these
/// // trat bounds. You do not have to use a custom enum.
/// #[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
/// enum MyCustomRectId {
///     RectOne,
///     RectTwo,
///     RectThree,
/// }
///
/// // A target bin ID just needs to meet these trait bounds (ideally also Copy)
/// // So you could use a u32, &str, or any other type that meets these
/// // trat bounds. You do not have to use a custom enum.
/// #[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
/// enum MyCustomBinId {
///     DestinationBinOne,
///     DestinationBinTwo,
/// }
///
/// // A placement group just needs to meet these trait bounds (ideally also Copy).
/// //
/// // Groups allow you to ensure that a set of rectangles will be placed
/// // into the same bin. If this isn't possible an error is returned.
/// //
/// // Groups are optional.
/// //
/// // You could use an i32, &'static str, or any other type that meets these
/// // trat bounds. You do not have to use a custom enum.
/// #[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
/// enum MyCustomGroupId {
///     GroupIdOne
/// }
///
/// let mut rects_to_place = GroupedRectsToPlace::new();
/// rects_to_place.push_rect(
///     MyCustomRectId::RectOne,
///     Some(vec![MyCustomGroupId::GroupIdOne]),
///     RectToInsert::new(10, 20, 255)
/// );
/// rects_to_place.push_rect(
///     MyCustomRectId::RectTwo,
///     Some(vec![MyCustomGroupId::GroupIdOne]),
///     RectToInsert::new(5, 50, 255)
/// );
/// rects_to_place.push_rect(
///     MyCustomRectId::RectThree,
///     None,
///     RectToInsert::new(30, 30, 255)
/// );
///
/// let mut target_bins = BTreeMap::new();
/// target_bins.insert(MyCustomBinId::DestinationBinOne, TargetBin::new(2048, 2048, 255));
/// target_bins.insert(MyCustomBinId::DestinationBinTwo, TargetBin::new(4096, 4096, 1020));
///
/// // Information about where each `MyCustomRectId` was placed
/// let rectangle_placements = pack_rects(
///     &rects_to_place,
///     &mut target_bins,
///     &volume_heuristic,
///     &contains_smallest_box
/// ).unwrap();
/// ```
///
/// ## Algorithm
///
/// The algorithm was originally inspired by [rectpack2D] and then modified to work in 3D.
///
/// [rectpack2D]: https://github.com/TeamHypersomnia/rectpack2D
///
/// ## TODO:
///
/// Optimize - plenty of room to remove clones and duplication .. etc
pub fn pack_rects<
    RectToPlaceId: Debug + Hash + PartialEq + Eq + Clone + Ord + PartialOrd,
    BinId: Debug + Hash + PartialEq + Eq + Clone + Ord + PartialOrd,
    GroupId: Debug + Hash + PartialEq + Eq + Clone + Ord + PartialOrd,
>(
    rects_to_place: &GroupedRectsToPlace<RectToPlaceId, GroupId>,
    target_bins: &mut BTreeMap<BinId, TargetBin>,
    box_size_heuristic: &BoxSizeHeuristicFn,
    more_suitable_containers_fn: &ComparePotentialContainersFn,
) -> Result<RectanglePackOk<RectToPlaceId, BinId>, RectanglePackError> {
    let mut packed_locations = KeyValMap::new();

    let mut target_bins: Vec<(&BinId, &mut TargetBin)> = target_bins.iter_mut().collect();
    sort_bins_smallest_to_largest(&mut target_bins, box_size_heuristic);

    let mut group_id_to_inbound_ids: Vec<(&Group<GroupId, RectToPlaceId>, &Vec<RectToPlaceId>)> =
        rects_to_place.group_id_to_inbound_ids.iter().collect();
    sort_groups_largest_to_smallest(
        &mut group_id_to_inbound_ids,
        rects_to_place,
        box_size_heuristic,
    );

    'group: for (_group_id, rects_to_place_ids) in group_id_to_inbound_ids {
        for (bin_id, bin) in target_bins.iter_mut() {
            if !can_fit_entire_group_into_bin(
                bin.clone(),
                &rects_to_place_ids[..],
                rects_to_place,
                box_size_heuristic,
                more_suitable_containers_fn,
            ) {
                continue;
            }

            'incoming: for rect_to_place_id in rects_to_place_ids.iter() {
                if bin.available_bin_sections.len() == 0 {
                    continue;
                }

                let _bin_clone = bin.clone();

                let mut bin_sections = bin.available_bin_sections.clone();

                let last_section_idx = bin_sections.len() - 1;
                let mut sections_tried = 0;

                'section: while let Some(remaining_section) = bin_sections.pop() {
                    let rect_to_place = rects_to_place.rects[&rect_to_place_id];

                    let placement = remaining_section.try_place(
                        &rect_to_place,
                        more_suitable_containers_fn,
                        box_size_heuristic,
                    );

                    if placement.is_err() {
                        sections_tried += 1;
                        continue 'section;
                    }

                    let (placement, mut new_sections) = placement.unwrap();
                    sort_by_size_largest_to_smallest(&mut new_sections, box_size_heuristic);

                    bin.remove_filled_section(last_section_idx - sections_tried);
                    bin.add_new_sections(new_sections);

                    packed_locations.insert(rect_to_place_id.clone(), (bin_id.clone(), placement));

                    continue 'incoming;
                }
            }

            continue 'group;
        }
        return Err(RectanglePackError::NotEnoughBinSpace);
    }

    Ok(RectanglePackOk { packed_locations })
}

// TODO: This is duplicative of the code above
fn can_fit_entire_group_into_bin<RectToPlaceId, GroupId>(
    mut bin: TargetBin,
    group: &[RectToPlaceId],
    rects_to_place: &GroupedRectsToPlace<RectToPlaceId, GroupId>,

    box_size_heuristic: &BoxSizeHeuristicFn,
    more_suitable_containers_fn: &ComparePotentialContainersFn,
) -> bool
where
    RectToPlaceId: Debug + Hash + PartialEq + Eq + Clone + Ord + PartialOrd,
    GroupId: Debug + Hash + PartialEq + Eq + Clone + Ord + PartialOrd,
{
    'incoming: for rect_to_place_id in group.iter() {
        if bin.available_bin_sections.len() == 0 {
            return false;
        }

        let mut bin_sections = bin.available_bin_sections.clone();

        let last_section_idx = bin_sections.len() - 1;
        let mut sections_tried = 0;

        'section: while let Some(remaining_section) = bin_sections.pop() {
            let rect_to_place = rects_to_place.rects[&rect_to_place_id];

            let placement = remaining_section.try_place(
                &rect_to_place,
                more_suitable_containers_fn,
                box_size_heuristic,
            );

            if placement.is_err() {
                sections_tried += 1;
                continue 'section;
            }

            let (_placement, mut new_sections) = placement.unwrap();
            sort_by_size_largest_to_smallest(&mut new_sections, box_size_heuristic);

            bin.remove_filled_section(last_section_idx - sections_tried);
            bin.add_new_sections(new_sections);

            continue 'incoming;
        }

        return false;
    }

    true
}

/// Information about successfully packed rectangles.
#[derive(Debug, PartialEq)]
pub struct RectanglePackOk<RectToPlaceId: PartialEq + Eq + Hash, BinId: PartialEq + Eq + Hash> {
    packed_locations: KeyValMap<RectToPlaceId, (BinId, PackedLocation)>,
    // TODO: Other information such as information about how the bins were packed
    // (perhaps percentage filled)
}

impl<RectToPlaceId: PartialEq + Eq + Hash, BinId: PartialEq + Eq + Hash>
    RectanglePackOk<RectToPlaceId, BinId>
{
    /// Indicates where every incoming rectangle was placed
    pub fn packed_locations(&self) -> &KeyValMap<RectToPlaceId, (BinId, PackedLocation)> {
        &self.packed_locations
    }
}

/// An error while attempting to pack rectangles into bins.
#[derive(Debug, PartialEq)]
pub enum RectanglePackError {
    /// The rectangles can't be placed into the bins. More bin space needs to be provided.
    NotEnoughBinSpace,
}

#[cfg(std)]
impl std::error::Error for RectanglePackError {}

impl Display for RectanglePackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            RectanglePackError::NotEnoughBinSpace => {
                f.write_str("Not enough space to place all of the rectangles.")
            }
        }
    }
}

fn sort_bins_smallest_to_largest<BinId>(
    bins: &mut Vec<(&BinId, &mut TargetBin)>,
    box_size_heuristic: &BoxSizeHeuristicFn,
) where
    BinId: Debug + Hash + PartialEq + Eq + Clone,
{
    bins.sort_by(|a, b| {
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
    items.sort_by(|a, b| box_size_heuristic(b.whd).cmp(&box_size_heuristic(a.whd)));
}

fn sort_groups_largest_to_smallest<GroupId, RectToPlaceId>(
    group_id_to_inbound_ids: &mut Vec<(&Group<GroupId, RectToPlaceId>, &Vec<RectToPlaceId>)>,
    incoming_groups: &GroupedRectsToPlace<RectToPlaceId, GroupId>,
    box_size_heuristic: &BoxSizeHeuristicFn,
) where
    RectToPlaceId: Debug + Hash + PartialEq + Eq + Clone + Ord + PartialOrd,
    GroupId: Debug + Hash + PartialEq + Eq + Clone + Ord + PartialOrd,
{
    group_id_to_inbound_ids.sort_by(|a, b| {
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

#[cfg(test)]
mod tests {
    use crate::{pack_rects, volume_heuristic, RectToInsert, RectanglePackError, TargetBin};

    use super::*;
    use crate::packed_location::RotatedBy;

    /// If the provided rectangles can't fit into the provided bins.
    #[test]
    fn error_if_the_rectangles_cannot_fit_into_target_bins() {
        let mut targets = BTreeMap::new();
        targets.insert(BinId::Three, TargetBin::new(2, 100, 1));

        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();
        groups.push_rect(RectToPlaceId::One, None, RectToInsert::new(3, 1, 1));

        match pack_rects(
            &groups,
            &mut targets,
            &volume_heuristic,
            &contains_smallest_box,
        )
        .unwrap_err()
        {
            RectanglePackError::NotEnoughBinSpace => {}
        };
    }

    /// Rectangles in the same group need to be placed in the same bin.
    ///
    /// Here we create two Rectangles in the same group and create two bins that could fit them
    /// individually but cannot fit them together.
    ///
    /// Then we verify that we receive an error for being unable to place the group.
    #[test]
    fn error_if_cannot_fit_group() {
        let mut targets = BTreeMap::new();
        targets.insert(BinId::Three, TargetBin::new(100, 100, 1));
        targets.insert(BinId::Four, TargetBin::new(100, 100, 1));

        let mut groups = GroupedRectsToPlace::new();
        groups.push_rect(
            RectToPlaceId::One,
            Some(vec!["A Group"]),
            RectToInsert::new(100, 100, 1),
        );
        groups.push_rect(
            RectToPlaceId::Two,
            Some(vec!["A Group"]),
            RectToInsert::new(100, 100, 1),
        );

        match pack_rects(
            &groups,
            &mut targets,
            &volume_heuristic,
            &contains_smallest_box,
        )
        .unwrap_err()
        {
            RectanglePackError::NotEnoughBinSpace => {}
        };
    }

    /// If we provide a single inbound rectangle and a single bin - it should be placed into that
    /// bin.
    #[test]
    fn one_inbound_rect_one_bin() {
        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();
        groups.push_rect(RectToPlaceId::One, None, RectToInsert::new(1, 2, 1));

        let mut targets = BTreeMap::new();
        targets.insert(BinId::Three, TargetBin::new(5, 5, 1));

        let packed = pack_rects(
            &groups,
            &mut targets,
            &volume_heuristic,
            &contains_smallest_box,
        )
        .unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 1);

        assert_eq!(locations[&RectToPlaceId::One].0, BinId::Three,);
        assert_eq!(
            locations[&RectToPlaceId::One].1,
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
        groups.push_rect(RectToPlaceId::One, None, RectToInsert::new(2, 2, 1));

        let mut targets = BTreeMap::new();
        targets.insert(BinId::Three, TargetBin::new(5, 5, 1));
        targets.insert(BinId::Four, TargetBin::new(5, 5, 2));

        let packed = pack_rects(
            &groups,
            &mut targets,
            &volume_heuristic,
            &contains_smallest_box,
        )
        .unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations[&RectToPlaceId::One].0, BinId::Three,);

        assert_eq!(locations.len(), 1);
        assert_eq!(
            locations[&RectToPlaceId::One].1,
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

    /// If we have two inbound rects the largest one should be placed first.
    #[test]
    fn places_largest_rectangles_first() {
        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();
        groups.push_rect(RectToPlaceId::One, None, RectToInsert::new(10, 10, 1));
        groups.push_rect(RectToPlaceId::Two, None, RectToInsert::new(5, 5, 1));

        let mut targets = BTreeMap::new();
        targets.insert(BinId::Three, TargetBin::new(20, 20, 2));

        let packed = pack_rects(
            &groups,
            &mut targets,
            &volume_heuristic,
            &contains_smallest_box,
        )
        .unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);

        assert_eq!(locations[&RectToPlaceId::One].0, BinId::Three,);
        assert_eq!(locations[&RectToPlaceId::Two].0, BinId::Three,);

        assert_eq!(
            locations[&RectToPlaceId::One].1,
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
            locations[&RectToPlaceId::Two].1,
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
    /// 2. Second place the remaining rectangle into the next available bin (i.e. the largest one).
    #[test]
    fn two_rects_two_bins() {
        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();
        groups.push_rect(RectToPlaceId::One, None, RectToInsert::new(15, 15, 1));
        groups.push_rect(RectToPlaceId::Two, None, RectToInsert::new(20, 20, 1));

        let mut targets = BTreeMap::new();
        targets.insert(BinId::Three, TargetBin::new(20, 20, 1));
        targets.insert(BinId::Four, TargetBin::new(50, 50, 1));

        let packed = pack_rects(
            &groups,
            &mut targets,
            &volume_heuristic,
            &contains_smallest_box,
        )
        .unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);

        assert_eq!(locations[&RectToPlaceId::One].0, BinId::Four,);
        assert_eq!(locations[&RectToPlaceId::Two].0, BinId::Three,);

        assert_eq!(
            locations[&RectToPlaceId::One].1,
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
            locations[&RectToPlaceId::Two].1,
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
        let mut targets = BTreeMap::new();
        targets.insert(BinId::Three, TargetBin::new(100, 100, 1));

        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();

        groups.push_rect(RectToPlaceId::One, None, RectToInsert::new(50, 90, 1));
        groups.push_rect(RectToPlaceId::Two, None, RectToInsert::new(1, 1, 1));

        let packed = pack_rects(
            &groups,
            &mut targets,
            &volume_heuristic,
            &contains_smallest_box,
        )
        .unwrap();
        let locations = packed.packed_locations;

        assert_eq!(locations.len(), 2);

        assert_eq!(locations[&RectToPlaceId::One].0, BinId::Three,);
        assert_eq!(locations[&RectToPlaceId::Two].0, BinId::Three,);

        assert_eq!(
            locations[&RectToPlaceId::One].1,
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
            locations[&RectToPlaceId::Two].1,
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
        let mut targets = BTreeMap::new();
        targets.insert(BinId::Three, TargetBin::new(100, 100, 1));

        let mut groups: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();

        groups.push_rect(RectToPlaceId::One, None, RectToInsert::new(60, 95, 1));
        groups.push_rect(RectToPlaceId::Two, None, RectToInsert::new(40, 10, 1));
        groups.push_rect(RectToPlaceId::Three, None, RectToInsert::new(60, 3, 1));

        let packed = pack_rects(
            &groups,
            &mut targets,
            &volume_heuristic,
            &contains_smallest_box,
        )
        .unwrap();
        let locations = packed.packed_locations;

        assert_eq!(
            locations[&RectToPlaceId::One].1,
            PackedLocation {
                x: 0,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 60,
                    height: 95,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
        assert_eq!(
            locations[&RectToPlaceId::Two].1,
            PackedLocation {
                x: 60,
                y: 0,
                z: 0,
                whd: WidthHeightDepth {
                    width: 40,
                    height: 10,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
        assert_eq!(
            locations[&RectToPlaceId::Three].1,
            PackedLocation {
                x: 0,
                y: 95,
                z: 0,
                whd: WidthHeightDepth {
                    width: 60,
                    height: 3,
                    depth: 1
                },
                x_axis_rotation: RotatedBy::ZeroDegrees,
                y_axis_rotation: RotatedBy::ZeroDegrees,
                z_axis_rotation: RotatedBy::ZeroDegrees,
            }
        );
    }

    /// Create a handful of rectangles that need to be placed, with two of them in the same group
    /// and the rest ungrouped.
    /// Try placing them many times and verify that each time they are placed the exact same way.
    #[test]
    fn deterministic_packing() {
        let mut previous_packed = None;

        for _ in 0..5 {
            let mut rects_to_place: GroupedRectsToPlace<&'static str, &str> =
                GroupedRectsToPlace::new();

            let mut target_bins = BTreeMap::new();
            for bin_id in 0..5 {
                target_bins.insert(bin_id, TargetBin::new(8, 8, 1));
            }

            let rectangles = vec![
                "some-rectangle-0",
                "some-rectangle-1",
                "some-rectangle-2",
                "some-rectangle-3",
                "some-rectangle-4",
            ];

            for rect_id in rectangles.iter() {
                rects_to_place.push_rect(rect_id, None, RectToInsert::new(4, 4, 1));
            }

            let packed = pack_rects(
                &rects_to_place,
                &mut target_bins.clone(),
                &volume_heuristic,
                &contains_smallest_box,
            )
            .unwrap();

            if let Some(previous_packed) = previous_packed.as_ref() {
                assert_eq!(&packed, previous_packed);
            }

            previous_packed = Some(packed);
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
    enum RectToPlaceId {
        One,
        Two,
        Three,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
    enum BinId {
        Three,
        Four,
    }
}
