use crate::bin_section::BinSection;

impl BinSection {
    /// Whether or not two bin sections overlap each other.
    pub fn overlaps(&self, other: &Self) -> bool {
        (self.x >= other.x && self.x <= other.right())
            && (self.y >= other.y && self.y <= other.top())
            && (self.z >= other.z && self.z <= other.back())
    }

    fn right(&self) -> u32 {
        self.x + (self.whd.width - 1)
    }

    fn top(&self) -> u32 {
        self.y + (self.whd.height - 1)
    }

    fn back(&self) -> u32 {
        self.z + (self.whd.depth - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width_height_depth::WidthHeightDepth;

    /// Verify that the overlaps method works properly.
    #[test]
    fn overlaps() {
        OverlapsTest {
            label: "Overlaps X, Y and Z",
            section1: BinSection::new(3, 4, 5, WidthHeightDepth::new(1, 1, 1)),
            section2: section_2_3_4(),
            expected_overlap: true,
        }
        .test();

        OverlapsTest {
            label: "Overlaps X only",
            section1: BinSection::new(3, 40, 50, WidthHeightDepth::new(1, 1, 1)),
            section2: section_2_3_4(),
            expected_overlap: false,
        }
        .test();

        OverlapsTest {
            label: "Overlaps Y only",
            section1: BinSection::new(30, 4, 50, WidthHeightDepth::new(1, 1, 1)),
            section2: section_2_3_4(),
            expected_overlap: false,
        }
        .test();

        OverlapsTest {
            label: "Overlaps Z only",
            section1: BinSection::new(30, 40, 5, WidthHeightDepth::new(1, 1, 1)),
            section2: section_2_3_4(),
            expected_overlap: false,
        }
        .test();
    }

    fn section_2_3_4() -> BinSection {
        BinSection::new(2, 3, 4, WidthHeightDepth::new(2, 3, 4))
    }

    struct OverlapsTest {
        label: &'static str,
        section1: BinSection,
        section2: BinSection,
        expected_overlap: bool,
    }

    impl OverlapsTest {
        fn test(self) {
            assert_eq!(
                self.section1.overlaps(&self.section2),
                self.expected_overlap,
                "{}",
                self.label
            )
        }
    }
}
