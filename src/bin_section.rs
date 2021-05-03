use crate::packed_location::RotatedBy;
use crate::{BoxSizeHeuristicFn, PackedLocation, RectToInsert, WidthHeightDepth};

use core::{
    cmp::Ordering,
    fmt::{Debug, Display, Error as FmtError, Formatter},
};

mod overlaps;

/// Given two sets of containers, which of these is the more suitable for our packing.
///
/// Useful when we're determining how to split up the remaining volume/area of a box/rectangle.
///
/// For example - we might deem it best to cut the remaining region vertically, or horizontally,
/// or along the Z-axis.
///
/// This decision is based on the more suitable contains heuristic. We determine all 6 possible
/// ways to divide up remaining space, sort them using the more suitable contains heuristic function
/// and choose the best one.
///
/// Ordering::Greater means the first set of containers is better.
/// Ordering::Less means the second set of containers is better.
pub type ComparePotentialContainersFn =
    dyn Fn([WidthHeightDepth; 3], [WidthHeightDepth; 3], &BoxSizeHeuristicFn) -> Ordering;

/// Select the container that has the smallest box.
///
/// If there is a tie on the smallest boxes, select whichever also has the second smallest box.
pub fn contains_smallest_box(
    mut container1: [WidthHeightDepth; 3],
    mut container2: [WidthHeightDepth; 3],
    heuristic: &BoxSizeHeuristicFn,
) -> Ordering {
    container1.sort_by(|a, b| heuristic(*a).cmp(&heuristic(*b)));
    container2.sort_by(|a, b| heuristic(*a).cmp(&heuristic(*b)));

    match heuristic(container2[0]).cmp(&heuristic(container1[0])) {
        Ordering::Equal => heuristic(container2[1]).cmp(&heuristic(container1[1])),
        o => o,
    }
}

/// A rectangular section within a target bin that takes up one or more layers
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default, Ord, PartialOrd)]
pub struct BinSection {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) z: u32,
    pub(crate) whd: WidthHeightDepth,
}

/// An error while attempting to place a rectangle within a bin section;
#[derive(Debug, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum BinSectionError {
    PlacementWiderThanBinSection,
    PlacementTallerThanBinSection,
    PlacementDeeperThanBinSection,
}

impl Display for BinSectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let err = match self {
            BinSectionError::PlacementWiderThanBinSection => {
                "Can not place a rectangle inside of a bin that is wider than that rectangle."
            }
            BinSectionError::PlacementTallerThanBinSection => {
                "Can not place a rectangle inside of a bin that is taller than that rectangle."
            }
            BinSectionError::PlacementDeeperThanBinSection => {
                "Can not place a rectangle inside of a bin that is deeper than that rectangle."
            }
        };

        f.write_str(err)
    }
}

impl BinSection {
    /// Create a new BinSection
    pub fn new(x: u32, y: u32, z: u32, whd: WidthHeightDepth) -> Self {
        BinSection { x, y, z, whd }
    }

    // TODO: Delete - just the old API before we had the WidthHeightDepth struct
    fn new_spread(x: u32, y: u32, z: u32, width: u32, height: u32, depth: u32) -> Self {
        BinSection {
            x,
            y,
            z,
            whd: WidthHeightDepth {
                width,
                height,
                depth,
            },
        }
    }
}

impl BinSection {
    /// See if a `LayeredRect` can fit inside of this BinSection.
    ///
    /// If it can we return the `BinSection`s that would be created by placing the `LayeredRect`
    /// inside of this `BinSection`.
    ///
    /// Consider the diagram below of a smaller box placed into of a larger one.
    ///
    /// The remaining space can be divided into three new sections.
    ///
    /// There are several ways to make this division.
    ///
    /// You could keep all of the space above the smaller box intact and split up the space
    /// behind and to the right of it.
    ///
    /// But within that you have a choice between whether the overlapping space goes to right
    /// or behind box.
    ///
    /// Or you could keep the space to the right and split the top and behind space.
    ///
    /// etc.
    ///
    /// There are six possible configurations of newly created sections. The configuration to use
    /// is decided on based on a a function provided by the consumer.
    ///
    ///
    /// ```text
    ///             ┌┬───────────────────┬┐
    ///           ┌─┘│                 ┌─┘│
    ///         ┌─┘  │               ┌─┘  │
    ///       ┌─┘    │             ┌─┘    │
    ///     ┌─┘      │           ┌─┘      │
    ///   ┌─┘        │         ┌─┘        │
    /// ┌─┴──────────┼───────┬─┘          │
    /// │            │       │            │
    /// │            │       │            │
    /// │       ┌┬───┴────┬─┐│            │
    /// │     ┌─┘│      ┌─┘ ││            │
    /// │   ┌─┘  │    ┌─┘   ││            │
    /// │ ┌─┘    │  ┌─┘     ├┼───────────┬┘
    /// ├─┴──────┤ ─┘       ││         ┌─┘
    /// │       ┌┴─┬───────┬┘│       ┌─┘   
    /// │     ┌─┘  │     ┌─┘ │     ┌─┘     
    /// │   ┌─┘    │   ┌─┘   │   ┌─┘       
    /// │ ┌─┘      │ ┌─┘     │ ┌─┘         
    /// └─┴────────┴─┴───────┴─┘           
    /// ```
    ///
    /// # Note
    ///
    /// Written to be readable/maintainable, not to minimize conditional logic, under the
    /// (unverified) assumption that a release compilation will inline and dedupe the function
    /// calls and conditionals.
    pub fn try_place(
        &self,
        incoming: &RectToInsert,
        container_comparison_fn: &ComparePotentialContainersFn,
        heuristic_fn: &BoxSizeHeuristicFn,
    ) -> Result<(PackedLocation, [BinSection; 3]), BinSectionError> {
        self.incoming_can_fit(incoming)?;

        let mut all_combinations = [
            self.depth_largest_height_second_largest_width_smallest(incoming),
            self.depth_largest_width_second_largest_height_smallest(incoming),
            self.height_largest_depth_second_largest_width_smallest(incoming),
            self.height_largest_width_second_largest_depth_smallest(incoming),
            self.width_largest_depth_second_largest_height_smallest(incoming),
            self.width_largest_height_second_largest_depth_smallest(incoming),
        ];

        all_combinations.sort_by(|a, b| {
            container_comparison_fn(
                [a[0].whd, a[1].whd, a[2].whd],
                [b[0].whd, b[1].whd, b[2].whd],
                heuristic_fn,
            )
        });

        let packed_location = PackedLocation {
            x: self.x,
            y: self.y,
            z: self.z,
            whd: WidthHeightDepth {
                width: incoming.width(),
                height: incoming.height(),
                depth: incoming.depth(),
            },
            x_axis_rotation: RotatedBy::ZeroDegrees,
            y_axis_rotation: RotatedBy::ZeroDegrees,
            z_axis_rotation: RotatedBy::ZeroDegrees,
        };

        Ok((packed_location, all_combinations[5]))
    }

    fn incoming_can_fit(&self, incoming: &RectToInsert) -> Result<(), BinSectionError> {
        if incoming.width() > self.whd.width {
            return Err(BinSectionError::PlacementWiderThanBinSection);
        }
        if incoming.height() > self.whd.height {
            return Err(BinSectionError::PlacementTallerThanBinSection);
        }

        if incoming.depth() > self.whd.depth {
            return Err(BinSectionError::PlacementDeeperThanBinSection);
        }

        Ok(())
    }

    fn width_largest_height_second_largest_depth_smallest(
        &self,
        incoming: &RectToInsert,
    ) -> [BinSection; 3] {
        [
            self.empty_space_directly_right(incoming),
            self.all_empty_space_above_excluding_behind(incoming),
            self.all_empty_space_behind(incoming),
        ]
    }

    fn width_largest_depth_second_largest_height_smallest(
        &self,
        incoming: &RectToInsert,
    ) -> [BinSection; 3] {
        [
            self.empty_space_directly_right(incoming),
            self.all_empty_space_above(incoming),
            self.all_empty_space_behind_excluding_above(incoming),
        ]
    }

    fn height_largest_width_second_largest_depth_smallest(
        &self,
        incoming: &RectToInsert,
    ) -> [BinSection; 3] {
        [
            self.all_empty_space_right_excluding_behind(incoming),
            self.empty_space_directly_above(incoming),
            self.all_empty_space_behind(incoming),
        ]
    }

    fn height_largest_depth_second_largest_width_smallest(
        &self,
        incoming: &RectToInsert,
    ) -> [BinSection; 3] {
        [
            self.all_empty_space_right(incoming),
            self.empty_space_directly_above(incoming),
            self.all_empty_space_behind_excluding_right(incoming),
        ]
    }

    fn depth_largest_width_second_largest_height_smallest(
        &self,
        incoming: &RectToInsert,
    ) -> [BinSection; 3] {
        [
            self.all_empty_space_right_excluding_above(incoming),
            self.all_empty_space_above(incoming),
            self.empty_space_directly_behind(incoming),
        ]
    }

    fn depth_largest_height_second_largest_width_smallest(
        &self,
        incoming: &RectToInsert,
    ) -> [BinSection; 3] {
        [
            self.all_empty_space_right(incoming),
            self.all_empty_space_above_excluding_right(incoming),
            self.empty_space_directly_behind(incoming),
        ]
    }

    fn all_empty_space_above(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new_spread(
            self.x,
            self.y + incoming.height(),
            self.z,
            self.whd.width,
            self.whd.height - incoming.height(),
            self.whd.depth,
        )
    }

    fn all_empty_space_right(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new_spread(
            self.x + incoming.width(),
            self.y,
            self.z,
            self.whd.width - incoming.width(),
            self.whd.height,
            self.whd.depth,
        )
    }

    fn all_empty_space_behind(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new_spread(
            self.x,
            self.y,
            self.z + incoming.depth(),
            self.whd.width,
            self.whd.height,
            self.whd.depth - incoming.depth(),
        )
    }

    fn empty_space_directly_above(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new_spread(
            self.x,
            self.y + incoming.height(),
            self.z,
            incoming.width(),
            self.whd.height - incoming.height(),
            incoming.depth(),
        )
    }

    fn empty_space_directly_right(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new_spread(
            self.x + incoming.width(),
            self.y,
            self.z,
            self.whd.width - incoming.width(),
            incoming.height(),
            incoming.depth(),
        )
    }

    fn empty_space_directly_behind(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new(
            self.x,
            self.y,
            self.z + incoming.depth(),
            WidthHeightDepth {
                width: incoming.width(),
                height: incoming.height(),
                depth: self.whd.depth - incoming.depth(),
            },
        )
    }

    fn all_empty_space_above_excluding_right(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new(
            self.x,
            self.y + incoming.height(),
            self.z,
            WidthHeightDepth {
                width: incoming.width(),
                height: self.whd.height - incoming.height(),
                depth: self.whd.depth,
            },
        )
    }

    fn all_empty_space_above_excluding_behind(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new(
            self.x,
            self.y + incoming.height(),
            self.z,
            WidthHeightDepth {
                width: self.whd.width,
                height: self.whd.height - incoming.height(),
                depth: incoming.depth(),
            },
        )
    }

    fn all_empty_space_right_excluding_above(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new(
            self.x + incoming.width(),
            self.y,
            self.z,
            WidthHeightDepth {
                width: self.whd.width - incoming.width(),
                height: incoming.height(),
                depth: self.whd.depth,
            },
        )
    }

    fn all_empty_space_right_excluding_behind(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new(
            self.x + incoming.width(),
            self.y,
            self.z,
            WidthHeightDepth {
                width: self.whd.width - incoming.width(),
                height: self.whd.height,
                depth: incoming.depth(),
            },
        )
    }

    fn all_empty_space_behind_excluding_above(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new(
            self.x,
            self.y,
            self.z + incoming.depth(),
            WidthHeightDepth {
                width: self.whd.width,
                height: incoming.height(),
                depth: self.whd.depth - incoming.depth(),
            },
        )
    }

    fn all_empty_space_behind_excluding_right(&self, incoming: &RectToInsert) -> BinSection {
        BinSection::new(
            self.x,
            self.y,
            self.z + incoming.depth(),
            WidthHeightDepth {
                width: incoming.width(),
                height: self.whd.height,
                depth: self.whd.depth - incoming.depth(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{volume_heuristic, RectToInsert};

    const BIGGEST: u32 = 50;
    const MIDDLE: u32 = 25;
    const SMALLEST: u32 = 10;

    const FULL: u32 = 100;

    /// If we're trying to place a rectangle that is wider than the container we return an error
    #[test]
    fn error_if_placement_is_wider_than_bin_section() {
        let bin_section = bin_section_width_height_depth(5, 20, 1);
        let placement = RectToInsert::new(6, 20, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &contains_smallest_box, &volume_heuristic)
                .unwrap_err(),
            BinSectionError::PlacementWiderThanBinSection
        );
    }

    /// If we're trying to place a rectangle that is taller than the container we return an error
    #[test]
    fn error_if_placement_is_taller_than_bin_section() {
        let bin_section = bin_section_width_height_depth(5, 20, 1);
        let placement = RectToInsert::new(5, 21, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &contains_smallest_box, &volume_heuristic)
                .unwrap_err(),
            BinSectionError::PlacementTallerThanBinSection
        );
    }

    /// If we're trying to place a rectangle that is deeper than the container we return an error
    #[test]
    fn error_if_placement_is_deeper_than_bin_section() {
        let bin_section = bin_section_width_height_depth(5, 20, 1);
        let placement = RectToInsert::new(5, 20, 2);

        assert_eq!(
            bin_section
                .try_place(&placement, &contains_smallest_box, &volume_heuristic)
                .unwrap_err(),
            BinSectionError::PlacementDeeperThanBinSection
        );
    }

    fn test_splits(
        container_dimensions: u32,
        rect_to_place: WidthHeightDepth,
        mut expected: [BinSection; 3],
    ) {
        let dim = container_dimensions;
        let bin_section = bin_section_width_height_depth(dim, dim, dim);

        let whd = rect_to_place;

        let placement = RectToInsert::new(whd.width, whd.height, whd.depth);

        let mut packed = bin_section
            .try_place(&placement, &contains_smallest_box, &volume_heuristic)
            .unwrap();

        packed.1.sort();
        expected.sort();

        assert_eq!(packed.1, expected);
    }

    /// Verify that we choose the correct splits when the placed rectangle is width > height > depth
    #[test]
    fn width_largest_height_second_largest_depth_smallest() {
        let whd = WidthHeightDepth {
            width: BIGGEST,
            height: MIDDLE,
            depth: SMALLEST,
        };

        test_splits(
            FULL,
            whd,
            [
                BinSection::new_spread(whd.width, 0, 0, FULL - whd.width, whd.height, whd.depth),
                BinSection::new_spread(0, whd.height, 0, FULL, FULL - whd.height, whd.depth),
                BinSection::new_spread(0, 0, whd.depth, FULL, FULL, FULL - whd.depth),
            ],
        );
    }

    /// Verify that we choose the correct splits when the placed rectangle is width > depth > height
    #[test]
    fn width_largest_depth_second_largest_height_smallest() {
        let whd = WidthHeightDepth {
            width: BIGGEST,
            height: SMALLEST,
            depth: MIDDLE,
        };

        test_splits(
            FULL,
            whd,
            [
                BinSection::new_spread(whd.width, 0, 0, FULL - whd.width, whd.height, whd.depth),
                BinSection::new_spread(0, whd.height, 0, FULL, FULL - whd.height, FULL),
                BinSection::new_spread(0, 0, whd.depth, FULL, whd.height, FULL - whd.depth),
            ],
        );
    }

    /// Verify that we choose the correct splits when the placed rectangle is height > width > depth
    #[test]
    fn height_largest_width_second_largest_depth_smallest() {
        let whd = WidthHeightDepth {
            width: MIDDLE,
            height: BIGGEST,
            depth: SMALLEST,
        };

        test_splits(
            FULL,
            whd,
            [
                BinSection::new_spread(whd.width, 0, 0, FULL - whd.width, FULL, whd.depth),
                BinSection::new_spread(0, whd.height, 0, whd.width, FULL - whd.height, whd.depth),
                BinSection::new_spread(0, 0, whd.depth, FULL, FULL, FULL - whd.depth),
            ],
        );
    }

    /// Verify that we choose the correct splits when the placed rectangle is height > depth > width
    #[test]
    fn height_largest_depth_second_largest_width_smallest() {
        let whd = WidthHeightDepth {
            width: SMALLEST,
            height: BIGGEST,
            depth: MIDDLE,
        };

        test_splits(
            FULL,
            whd,
            [
                BinSection::new_spread(whd.width, 0, 0, FULL - whd.width, FULL, FULL),
                BinSection::new_spread(0, whd.height, 0, whd.width, FULL - whd.height, whd.depth),
                BinSection::new_spread(0, 0, whd.depth, whd.width, FULL, FULL - whd.depth),
            ],
        );
    }

    /// Verify that we choose the correct splits when the placed rectangle is depth > width > height
    #[test]
    fn depth_largest_width_second_largest_height_smallest() {
        let whd = WidthHeightDepth {
            width: MIDDLE,
            height: SMALLEST,
            depth: BIGGEST,
        };

        test_splits(
            FULL,
            whd,
            [
                BinSection::new_spread(whd.width, 0, 0, FULL - whd.width, whd.height, FULL),
                BinSection::new_spread(0, whd.height, 0, FULL, FULL - whd.height, FULL),
                BinSection::new_spread(0, 0, whd.depth, whd.width, whd.height, FULL - whd.depth),
            ],
        );
    }

    /// Verify that we choose the correct splits when the placed rectangle is depth > height > width
    #[test]
    fn depth_largest_height_second_largest_width_smallest() {
        let whd = WidthHeightDepth {
            width: SMALLEST,
            height: MIDDLE,
            depth: BIGGEST,
        };

        test_splits(
            FULL,
            whd,
            [
                BinSection::new_spread(whd.width, 0, 0, FULL - whd.width, FULL, FULL),
                BinSection::new_spread(0, whd.height, 0, whd.width, FULL - whd.height, FULL),
                BinSection::new_spread(0, 0, whd.depth, whd.width, whd.height, FULL - whd.depth),
            ],
        );
    }

    // #[test]
    // fn todo() {
    //    unimplemented!("Add tests for supporting rotation");
    // }

    fn bin_section_width_height_depth(width: u32, height: u32, depth: u32) -> BinSection {
        BinSection::new(
            0,
            0,
            0,
            WidthHeightDepth {
                width,
                height,
                depth,
            },
        )
    }
}
