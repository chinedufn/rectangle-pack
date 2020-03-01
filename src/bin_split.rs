use crate::LayeredRect;

/// A rectangular section within a target bin that takes up one or more layers
#[derive(Debug, Eq, PartialEq)]
pub struct BinSection {
    x: u32,
    y_rel_bottom: u32,
    width: u32,
    height: u32,
    first_layer: u32,
    layer_count: u32,
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
    pub fn try_place(
        &self,
        incoming: &LayeredRect,
    ) -> Result<NewEmptyBinSections, BinSectionError> {
        if incoming.width() > self.width {
            unimplemented!()
        }

        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LayeredRect;

    /// If we're trying to place a rectangle that is wider than the container we return an error
    #[test]
    fn error_if_placement_is_wider_than_bin_section() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(6, 20, 1);

        assert_eq!(
            bin_section.try_place(&placement).unwrap_err(),
            BinSectionError::PlacementWiderThanBinSection
        );
    }

    /// If we're trying to place a rectangle that is taller than the container we return an error
    #[test]
    fn error_if_placement_is_taller_than_bin_section() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(5, 21, 1);

        assert_eq!(
            bin_section.try_place(&placement).unwrap_err(),
            BinSectionError::PlacementTallerThanBinSection
        );
    }

    /// If we're trying to place a rectangle that has more layers than the container we return an
    /// error
    #[test]
    fn error_if_placement_has_more_players_than_bin_section() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(5, 20, 2);

        assert_eq!(
            bin_section.try_place(&placement).unwrap_err(),
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
            bin_section.try_place(&placement).unwrap(),
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
            bin_section.try_place(&placement).unwrap(),
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
            bin_section.try_place(&placement).unwrap(),
            NewEmptyBinSections::One(BinSection::new(2, 0, 3, 20, 0, 1))
        );
    }

    /// If we place an inbound rectangle that is smaller than the target bin section in both width
    /// and height, two new splits should be created.
    #[test]
    fn smaller_width_and_height_placement_produces_two_sections() {
        let bin_section = bin_section_width_height(5, 20);
        let placement = LayeredRect::new(2, 9, 1);

        assert_eq!(
            bin_section.try_place(&placement).unwrap(),
            NewEmptyBinSections::Two([
                BinSection::new(2, 0, 3, 20, 0, 1),
                BinSection::new(0, 9, 5, 11, 0, 1),
            ])
        );
    }

    /// If we place an inbound rectangle of the same width/height as the target bin section but
    /// with fewer layers, one new section should be created.
    #[test]
    fn fewer_layers_produces_one_new_section() {
        let bin_section = bin_section_width_height_layer_count(5, 20, 5);
        let placement = LayeredRect::new(5, 20, 3);

        assert_eq!(
            bin_section.try_place(&placement).unwrap(),
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
            bin_section.try_place(&placement).unwrap(),
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
            bin_section.try_place(&placement).unwrap(),
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
            bin_section.try_place(&placement).unwrap(),
            NewEmptyBinSections::Three([
                BinSection::new(4, 0, 1, 20, 0, 3),
                BinSection::new(0, 9, 5, 11, 0, 3),
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
            bin_section.try_place(&placement).unwrap(),
            NewEmptyBinSections::Two([
                BinSection::new(0, 20, 50, 80, 0, 1),
                BinSection::new(40, 0, 10, 100, 0, 1),
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
            bin_section.try_place(&placement).unwrap(),
            NewEmptyBinSections::Two([
                BinSection::new(20, 0, 80, 50, 0, 1),
                BinSection::new(0, 40, 100, 10, 0, 1),
            ])
        );
    }

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
