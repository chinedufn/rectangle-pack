use crate::{HeuristicFn, LayeredRect, WidthHeightDepth};
use std::hint::unreachable_unchecked;

/// A rectangular section within a target bin that takes up one or more layers
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct BinSection {
    x: u32,
    y_rel_bottom: u32,
    width: u32,
    height: u32,
    first_layer: u32,
    layer_count: u32,
}

impl Into<WidthHeightDepth> for BinSection {
    fn into(self) -> WidthHeightDepth {
        WidthHeightDepth {
            width: self.width,
            height: self.height,
            depth: self.layer_count,
        }
    }
}

/// An error while attempting to place a rectangle within a bin section;
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum BinSectionError {
    #[error("Can not place a rectangle inside of a bin that is wider than that rectangle.")]
    PlacementWiderThanBinSection,
    #[error("Can not place a rectangle inside of a bin that is taller than that rectangle.")]
    PlacementTallerThanBinSection,
    #[error("Can not place a rectangle inside of a bin that has more layers than that rectangle.")]
    PlacementHasMoreLayersThanBinSection,
}

/// Bin sections that were created by splitting another bin section
#[derive(Debug, Eq, PartialEq)]
pub enum NewEmptyBinSections {
    /// The placed `LayeredRect` was the same size as the `BinSection`, so no new splits were
    /// created.
    None,
    /// The placed `LayeredRect` was smaller than the `BinSection` along one dimension,
    /// so one new split were created.
    One(BinSection),
    /// The placed `LayeredRect` was smaller than the `BinSection` along two dimensions,
    /// so one new split were created.
    Two([BinSection; 2]),
    /// The placed `LayeredRect` was smaller than the `BinSection` along the width, height and layer
    /// dimensions, so three news split were created.
    Three([BinSection; 3]),
}

impl BinSection {
    /// Create a new BinSection
    ///
    /// # Panics
    ///
    /// Panics if the layer_count == 0 since that would mean we're trying to make a section out of
    /// nothing.
    pub fn new(
        x: u32,
        y_rel_bottom: u32,
        width: u32,
        height: u32,
        first_layer: u32,
        layer_count: u32,
    ) -> Self {
        assert!(layer_count > 0);

        BinSection {
            x,
            y_rel_bottom,
            width,
            height,
            first_layer,
            layer_count,
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
        incoming: &LayeredRect,
        heuristic: &HeuristicFn,
    ) -> Result<NewEmptyBinSections, BinSectionError> {
        self.incoming_can_fit(incoming)?;

        if self.same_size(incoming) {
            return Ok(NewEmptyBinSections::None);
        }

        if self.same_width_same_layers_different_height(incoming) {
            let empty_space_above = self.all_empty_space_above(incoming);
            return Ok(NewEmptyBinSections::One(empty_space_above));
        }

        if self.same_height_same_layers_different_width(incoming) {
            let empty_space_right = self.all_empty_space_right(incoming);
            return Ok(NewEmptyBinSections::One(empty_space_right));
        }

        if self.same_width_same_height_fewer_layers(incoming) {
            let all_empty_space_behind = self.all_empty_space_behind(incoming);
            return Ok(NewEmptyBinSections::One(all_empty_space_behind));
        }

        if self.different_width_different_height_same_layers(incoming) {
            let splits = self.choose_largest_delta_xy_split(incoming, heuristic);
            return Ok(NewEmptyBinSections::Two(splits));
        }

        if self.same_height_different_width_fewer_layers(incoming) {
            return Ok(NewEmptyBinSections::Two([
                self.all_empty_space_right(incoming),
                self.all_empty_space_behind(incoming),
            ]));
        }

        if self.same_width_different_height_fewer_layers(incoming) {
            return Ok(NewEmptyBinSections::Two([
                self.all_empty_space_above(incoming),
                self.all_empty_space_behind(incoming),
            ]));
        }

        if self.different_height_different_with_fewer_layers(incoming) {
            let splits = self.choose_largest_delta_xy_split(incoming, heuristic);

            return Ok(NewEmptyBinSections::Three([
                splits[0],
                splits[1],
                self.all_empty_space_behind(incoming),
            ]));
        }

        // Safe because every possible combination of size differences is checked above.
        unsafe { unreachable_unchecked() }
    }

    fn incoming_can_fit(&self, incoming: &LayeredRect) -> Result<(), BinSectionError> {
        if incoming.width() > self.width {
            return Err(BinSectionError::PlacementWiderThanBinSection);
        }

        if incoming.height() > self.height {
            return Err(BinSectionError::PlacementTallerThanBinSection);
        }

        if incoming.layers() > self.layer_count {
            return Err(BinSectionError::PlacementHasMoreLayersThanBinSection);
        }

        Ok(())
    }

    fn same_size(&self, incoming: &LayeredRect) -> bool {
        incoming.width() == self.width
            && incoming.height() == self.height
            && incoming.layers() == self.layer_count
    }

    fn choose_largest_delta_xy_split(
        &self,
        incoming: &LayeredRect,
        heuristic: &dyn Fn(WidthHeightDepth) -> u128,
    ) -> [BinSection; 2] {
        let split_candidate_1 = [
            self.all_empty_space_above(incoming),
            self.empty_space_directly_right(incoming),
        ];

        let split_candidate_2 = [
            self.all_empty_space_right(incoming),
            self.empty_space_directly_above(incoming),
        ];

        let delta1 = heuristic(split_candidate_1[0].into()) as i128
            - heuristic(split_candidate_1[1].into()) as i128;
        let delta1 = delta1.abs();

        let delta2 = heuristic(split_candidate_2[0].into()) as i128
            - heuristic(split_candidate_2[1].into()) as i128;
        let delta2 = delta2.abs();

        match delta1 > delta2 {
            true => split_candidate_1,
            false => split_candidate_2,
        }
    }

    fn same_width_same_layers_different_height(&self, incoming: &LayeredRect) -> bool {
        incoming.width() == self.width
            && incoming.layers() == self.layer_count
            && incoming.height() != self.height
    }

    fn same_height_same_layers_different_width(&self, incoming: &LayeredRect) -> bool {
        incoming.height() == self.height
            && incoming.layers() == self.layer_count
            && incoming.width() != self.width
    }

    fn different_width_different_height_same_layers(&self, incoming: &LayeredRect) -> bool {
        incoming.width() != self.width
            && incoming.height() != self.height
            && incoming.layers() == self.layer_count
    }

    fn same_width_same_height_fewer_layers(&self, incoming: &LayeredRect) -> bool {
        incoming.width() == self.width
            && incoming.height() == self.height
            && incoming.layers() != self.layer_count
    }

    fn same_height_different_width_fewer_layers(&self, incoming: &LayeredRect) -> bool {
        incoming.height() == self.height
            && incoming.width() != self.width
            && incoming.layers() != self.layer_count
    }

    fn same_width_different_height_fewer_layers(&self, incoming: &LayeredRect) -> bool {
        incoming.width() == self.width
            && incoming.height() != self.height
            && incoming.layers() != self.layer_count
    }

    fn different_height_different_with_fewer_layers(&self, incoming: &LayeredRect) -> bool {
        incoming.height() != self.height
            && incoming.width() != self.width
            && incoming.layers() != self.layer_count
    }

    fn all_empty_space_above(&self, incoming: &LayeredRect) -> BinSection {
        BinSection::new(
            self.x,
            self.y_rel_bottom + incoming.height(),
            self.width,
            self.height - incoming.height(),
            self.first_layer,
            incoming.layers(),
        )
    }

    fn all_empty_space_right(&self, incoming: &LayeredRect) -> BinSection {
        BinSection::new(
            self.x + incoming.width(),
            self.y_rel_bottom,
            self.width - incoming.width(),
            self.height,
            self.first_layer,
            incoming.layers(),
        )
    }

    fn all_empty_space_behind(&self, incoming: &LayeredRect) -> BinSection {
        BinSection::new(
            self.x,
            self.y_rel_bottom,
            self.width,
            self.height,
            self.first_layer + incoming.layers(),
            self.layer_count - incoming.layers(),
        )
    }

    fn empty_space_directly_above(&self, incoming: &LayeredRect) -> BinSection {
        BinSection::new(
            self.x,
            self.y_rel_bottom + incoming.height(),
            incoming.width(),
            self.height - incoming.height(),
            self.first_layer,
            incoming.layers(),
        )
    }

    fn empty_space_directly_right(&self, incoming: &LayeredRect) -> BinSection {
        BinSection::new(
            self.x + incoming.width(),
            self.y_rel_bottom,
            self.width - incoming.width(),
            incoming.height(),
            self.first_layer,
            incoming.layers(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{volume_heuristic, LayeredRect};

    /// If we're trying to place a rectangle that is wider than the container we return an error
    #[test]
    fn error_if_placement_is_wider_than_bin_section() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(6, 20, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap_err(),
            BinSectionError::PlacementWiderThanBinSection
        );
    }

    /// If we're trying to place a rectangle that is taller than the container we return an error
    #[test]
    fn error_if_placement_is_taller_than_bin_section() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(5, 21, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap_err(),
            BinSectionError::PlacementTallerThanBinSection
        );
    }

    /// If we're trying to place a rectangle that has more layers than the container we return an
    /// error
    #[test]
    fn error_if_placement_has_more_layers_than_bin_section() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(5, 20, 2);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap_err(),
            BinSectionError::PlacementHasMoreLayersThanBinSection
        );
    }

    /// If we place an inbound rectangle on top of a bin section that it the same size, no new bin
    /// sections are generated
    #[test]
    fn placement_same_size_as_section_does_not_produce_new_section() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(5, 20, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::None
        );
    }

    /// If we place an inbound rectangle on top of a bin section that has the same width but a
    /// different height, only one new section is generated.
    #[test]
    fn placement_same_width_as_section_produces_one_new_section() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(5, 8, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::One(BinSection::new(0, 8, 5, 12, 0, 1))
        );
    }

    /// If we place an inbound rectangle on top of a bin section that has the same height but a
    /// different width, only one new section is generated.
    #[test]
    fn placement_same_height_as_section_produces_one_new_section() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(2, 20, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::One(BinSection::new(2, 0, 3, 20, 0, 1))
        );
    }

    /// If we place an inbound rectangle of the same width/height as the target bin section but
    /// with fewer layers, one new section should be created.
    #[test]
    fn fewer_layers_produces_one_new_section() {
        let bin_section = bin_section_width_height_layer_count(5, 20, 5);
        let placement = LayeredRect::new(5, 20, 3);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::One(BinSection::new(0, 0, 5, 20, 3, 2))
        );
    }

    /// If we place an inbound rectangle with less layers and width than the target bin section
    /// we produce two new empty sections.
    #[test]
    fn smaller_layers_smaller_width_produces_two_new_sections() {
        let bin_section = bin_section_width_height_layer_count(5, 20, 5);
        let placement = LayeredRect::new(4, 20, 3);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::Two([
                BinSection::new(4, 0, 1, 20, 0, 3),
                BinSection::new(0, 0, 5, 20, 3, 2),
            ])
        );
    }

    /// If we place an inbound rectangle with less layers and height than the target bin section
    /// we produce two new empty sections.
    #[test]
    fn smaller_layers_smaller_height_produces_two_new_sections() {
        let bin_section = bin_section_width_height_layer_count(5, 20, 5);
        let placement = LayeredRect::new(5, 9, 3);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::Two([
                BinSection::new(0, 9, 5, 11, 0, 3),
                BinSection::new(0, 0, 5, 20, 3, 2),
            ])
        );
    }

    /// If we place an inbound rectangle with less layers, width and height than the target bin
    /// section we produce three new empty sections.
    #[test]
    fn smaller_layers_smaller_width_smaller_height_produces_three_new_sections() {
        let bin_section = bin_section_width_height_layer_count(5, 20, 5);
        let placement = LayeredRect::new(4, 9, 3);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::Three([
                BinSection::new(0, 9, 5, 11, 0, 3),
                BinSection::new(4, 0, 1, 9, 0, 3),
                BinSection::new(0, 0, 5, 20, 3, 2),
            ])
        );
    }

    /// Verify that we split the remaining space horizontally in order to create a combination of
    /// two splits where one is as large as possible and the other is as small as possible.
    ///
    /// In general - large spaces are more usable and small spaces are less wasteful if they go
    /// unused.
    ///
    /// ```text
    /// ┌─────────────────────┐            
    /// │                     │            
    /// │                     │            
    /// │                     │            
    /// │                     │            
    /// │                     │            
    /// ├────────────────┬────▶ Horizontal
    /// │                │    │   Split    
    /// │   Placed Rect  │    │            
    /// │                │    │            
    /// └────────────────┴────┘            
    /// ```
    #[test]
    fn splits_horizontally_to_create_largest_possible_bin_split() {
        let bin_section = bin_section_width_height_layer_count(50, 100, 1);
        let placement = LayeredRect::new(40, 20, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::Two([
                BinSection::new(0, 20, 50, 80, 0, 1),
                BinSection::new(40, 0, 10, 20, 0, 1),
            ])
        );
    }

    /// Same as `#[test] splits_horizontally_to_create_largest_possible_bin_split` but with a third
    /// full empty section behind.
    #[test]
    fn splits_horizontally_to_create_largest_possible_bin_split_multi_layered() {
        let bin_section = bin_section_width_height_layer_count(50, 100, 3);
        let placement = LayeredRect::new(40, 20, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::Three([
                BinSection::new(0, 20, 50, 80, 0, 1),
                BinSection::new(40, 0, 10, 20, 0, 1),
                BinSection::new(0, 0, 50, 100, 1, 2)
            ])
        );
    }

    /// Verify that we split the remaining space vertically in order to create a combination of
    /// two splits where one is as large as possible and the other is as small as possible.
    ///
    /// In general - large spaces are more usable and small spaces are less wasteful if they go
    /// unused.
    ///
    /// ```text
    ///               Vertical                        
    ///                Split                          
    /// ┌────────────────▲──────────────┐
    /// ├────────────────┤              │
    /// │                │              │
    /// │   Placed Rect  │              │
    /// │                │              │
    /// └────────────────┴──────────────┘
    /// ```
    #[test]
    fn splits_vertically_to_create_largest_possible_bin_split() {
        let bin_section = bin_section_width_height_layer_count(100, 50, 1);
        let placement = LayeredRect::new(20, 40, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::Two([
                BinSection::new(20, 0, 80, 50, 0, 1),
                BinSection::new(0, 40, 20, 10, 0, 1),
            ])
        );
    }

    /// Same as `#[test] splits_vertically_to_create_largest_possible_bin_split` but with a third
    /// full empty section behind.
    #[test]
    fn splits_vertically_to_create_largest_possible_bin_split_multi_layered() {
        let bin_section = bin_section_width_height_layer_count(100, 50, 3);
        let placement = LayeredRect::new(20, 40, 1);

        assert_eq!(
            bin_section
                .try_place(&placement, &volume_heuristic)
                .unwrap(),
            NewEmptyBinSections::Three([
                BinSection::new(20, 0, 80, 50, 0, 1),
                BinSection::new(0, 40, 20, 10, 0, 1),
                BinSection::new(0, 0, 100, 50, 1, 2)
            ])
        );
    }

    #[test]
    fn add_tests_for_all_6_split_scenarios() {
        unimplemented!()
    }

    // -------
    // Trying out tests where we just have 6 tests, one for each potential split variant
    // -------

    fn bin_section_width_height(width: u32, height: u32) -> BinSection {
        BinSection::new(0, 0, width, height, 0, 1)
    }

    fn bin_section_width_height_layer_count(
        width: u32,
        height: u32,
        layer_count: u32,
    ) -> BinSection {
        BinSection::new(0, 0, width, height, 0, layer_count)
    }
}
