//! Methods for adding a BinSection back into a TargetBin.
//!
//! Useful in an application that needs to be able to remove packed rectangles from bins.
//! After which the [`TargetBin.coalesce`] method can be used to combine smaller adjacent sections
//! into larger sections.

#![allow(missing_docs)]

use crate::bin_section::BinSection;
use crate::TargetBin;
use core::fmt::{Display, Formatter, Result as FmtResult};

impl TargetBin {
    /// Push a [`BinSection`] to the list of remaining [`BinSection`]'s that rectangles can be
    /// placed in.
    ///
    /// ## Performance
    ///
    /// This checks that your [`BinSection`] does not overlap any other bin sections. In many
    /// cases this will be negligible, however it is important to note that this has a worst case
    /// time complexity of `O(Width * Height * Depth)`, where the worst case is tht you have a bin
    /// full of `1x1x1` rectangles.
    ///
    /// To skip the validity checks use [`TargetBin.push_available_bin_section_unchecked`].
    ///
    /// [`TargetBin.push_available_bin_section_unchecked`]: #method.push_available_bin_section_unchecked
    pub fn push_available_bin_section(
        &mut self,
        bin_section: BinSection,
    ) -> Result<(), PushBinSectionError> {
        if bin_section.x >= self.max_width
            || bin_section.y >= self.max_height
            || bin_section.z >= self.max_depth
        {
            return Err(PushBinSectionError::OutOfBounds(bin_section));
        }

        for available in self.available_bin_sections.iter() {
            if available.overlaps(&bin_section) {
                return Err(PushBinSectionError::Overlaps {
                    remaining_section: *available,
                    new_section: bin_section,
                });
            }
        }

        self.push_available_bin_section_unchecked(bin_section);

        Ok(())
    }

    /// Push a [`BinSection`] to the list of remaining [`BinSection`]'s that rectangles can be
    /// placed in, without checking whether or not it is valid.
    ///
    /// Use [`TargetBin.push_available_bin_section`] if you want to check that the new bin section
    /// does not overlap any existing bin sections nad that it is within the [`TargetBin`]'s bounds.
    ///
    /// [`TargetBin.push_available_bin_section`]: #method.push_available_bin_section
    pub fn push_available_bin_section_unchecked(&mut self, bin_section: BinSection) {
        self.available_bin_sections.push(bin_section);
    }
}

/// An error while attempting to push a [`BinSection`] into the remaining bin sections of a
/// [`TargetBin`].
#[derive(Debug)]
pub enum PushBinSectionError {
    /// Attempted to push a [`BinSection`] that is not fully contained by the bin.
    OutOfBounds(BinSection),
    /// Attempted to push a [`BinSection`] that overlaps another empty bin section.
    Overlaps {
        /// The section that is already stored as empty within the [`TargetBin`];
        remaining_section: BinSection,
        /// The section that you were trying to add to the [`TargetBin`];
        new_section: BinSection,
    },
}

impl Display for PushBinSectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PushBinSectionError::OutOfBounds(oob) => {
                f.debug_tuple("BinSection").field(oob).finish()
            }
            PushBinSectionError::Overlaps {
                remaining_section,
                new_section,
            } => f
                .debug_struct("Overlaps")
                .field("remaining_section", remaining_section)
                .field("new_section", new_section)
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width_height_depth::WidthHeightDepth;

    /// Verify that if the bin section that we are pushing is outside of the TargetBin's bounds we
    /// return an error.
    #[test]
    fn error_if_bin_section_out_of_bounds() {
        let mut bin = empty_bin();

        let out_of_bounds = BinSection::new(101, 0, 0, WidthHeightDepth::new(1, 1, 1));

        match bin.push_available_bin_section(out_of_bounds).err().unwrap() {
            PushBinSectionError::OutOfBounds(err_bin_section) => {
                assert_eq!(err_bin_section, out_of_bounds)
            }
            _ => panic!(),
        };
    }

    /// Verify that if the bin section that we are pushing overlaps another bin section we return
    /// an error.
    #[test]
    fn error_if_bin_section_overlaps_another_remaining_section() {
        let mut bin = empty_bin();

        let overlaps = BinSection::new(0, 0, 0, WidthHeightDepth::new(1, 1, 1));

        match bin.push_available_bin_section(overlaps).err().unwrap() {
            PushBinSectionError::Overlaps {
                remaining_section: err_remaining_section,
                new_section: err_new_section,
            } => {
                assert_eq!(err_new_section, overlaps);
                assert_eq!(
                    err_remaining_section,
                    BinSection::new(0, 0, 0, WidthHeightDepth::new(100, 100, 1))
                );
            }
            _ => panic!(),
        }
    }

    /// Verify that we can push a valid bin section.
    #[test]
    fn push_bin_section() {
        let mut bin = full_bin();

        let valid_section = BinSection::new(1, 2, 0, WidthHeightDepth::new(1, 1, 1));

        assert_eq!(bin.available_bin_sections.len(), 0);
        bin.push_available_bin_section(valid_section).unwrap();
        assert_eq!(bin.available_bin_sections.len(), 1);

        assert_eq!(bin.available_bin_sections[0], valid_section);
    }

    fn empty_bin() -> TargetBin {
        TargetBin::new(100, 100, 1)
    }

    fn full_bin() -> TargetBin {
        let mut bin = TargetBin::new(100, 100, 1);

        bin.available_bin_sections.clear();

        bin
    }
}
