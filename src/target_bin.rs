use crate::bin_section::BinSection;
use crate::width_height_depth::WidthHeightDepth;
use alloc::vec::Vec;

mod coalesce;
mod push_available_bin_section;

/// A bin that we'd like to play our incoming rectangles into
#[derive(Debug, Clone)]
pub struct TargetBin {
    pub(crate) max_width: u32,
    pub(crate) max_height: u32,
    pub(crate) max_depth: u32,
    pub(crate) available_bin_sections: Vec<BinSection>,
}

impl TargetBin {
    #[allow(missing_docs)]
    pub fn new(max_width: u32, max_height: u32, max_depth: u32) -> Self {
        let available_bin_sections = vec![BinSection::new(
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
            available_bin_sections,
        }
    }

    /// The free [`BinSection`]s within the [`TargetBin`] that rectangles can still be placed into.
    pub fn available_bin_sections(&self) -> &Vec<BinSection> {
        &self.available_bin_sections
    }

    /// Remove the section that was just split by a placed rectangle.
    pub fn remove_filled_section(&mut self, idx: usize) {
        self.available_bin_sections.remove(idx);
    }

    /// When a section is filled it gets split into three new sections.
    /// Here we add those.
    ///
    /// TODO: Ignore sections with a volume of 0
    pub fn add_new_sections(&mut self, new_sections: [BinSection; 3]) {
        for new_section in new_sections.iter() {
            if new_section.whd.volume() > 0 {
                self.available_bin_sections.push(*new_section);
            }
        }
    }
}
