use crate::bin_section::BinSection;
use crate::width_height_depth::WidthHeightDepth;

/// A bin that we'd like to play our incoming rectangles into
#[derive(Debug, Clone)]
pub struct TargetBin {
    pub(crate) max_width: u32,
    pub(crate) max_height: u32,
    pub(crate) max_depth: u32,
    pub(crate) remaining_sections: Vec<BinSection>,
}

impl TargetBin {
    #[allow(missing_docs)]
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

    /// Remove the section that was just split by a placed rectangle.
    pub fn remove_filled_section(&mut self) {
        self.remaining_sections.pop();
    }

    /// When a section is filled it gets split into three new sections.
    /// Here we add those.
    ///
    /// TODO: Ignore sections with a volume of 0
    pub fn add_new_sections(&mut self, new_sections: [BinSection; 3]) {
        for new_section in new_sections.iter() {
            self.remaining_sections.push(*new_section);
        }
    }
}
