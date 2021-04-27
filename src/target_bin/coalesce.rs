// Coalescing needs to be able to be done in pieces so that we can split it up over many frames.
//
// So, we can choose to coalesce a specific bin section, i.e. the 3rd bin section
// Then we can choose a range of other bin sections to try it against, so maybe 2..7
//
// Then next frame we can do 3rd bin section with 7..20
//
// Then when that's done we can move on to the 4th bin section
//
// So first we'll provide the raw function to take a bin section index and the range of
// other indices to try. Then we can provide any convenience methods that are necessary.
//
// We can introduce this behavior in two commits. One for adding back sections, the other
// for coalescing. Each can happen in a different module.

use crate::TargetBin;
use std::ops::Range;

impl TargetBin {
    /// Over time as you use [`TargetBin.push_available_bin_section`] to return remove packed
    /// rectangles from the [`TargetBin`], you may end up with neighboring bin sections that can
    /// be combined into a larger bin section.
    ///
    /// Combining bin sections in this was is desirable because a larger bin section allows you to
    /// place larger rectangles that might not fit into the smaller bin sections.
    ///
    /// In order to coalesce, or combine a bin section with other bin sections, we need to check
    /// every other available bin section to see if they are neighbors.
    ///
    /// This means that fully coalescing the entire list of available bin sections is O(n^2) time
    /// complexity, where n is the number of available empty sections.
    ///
    /// # Basic Usage
    ///
    /// ```
    /// # use rectangle_pack::TargetBin;
    /// let target_bin = my_target_bin();
    ///
    /// for idx in 0..target_bin.available_bin_sections().len() {
    ///     let len = target_bin.available_bin_sections().len();
    ///     target_bin.coalesce_available_sections(idx, 0..len);
    /// }
    ///
    /// # fn my_target_bin () -> TargetBin {
    /// #     TargetBin::new(1, 2, 3)
    /// # }
    /// ```
    ///
    /// # Distributing the Workload
    ///
    /// It is possible that you are developing an application that can in some cases have a lot of
    /// heavily fragmented bins that need to be coalesced. If your application has a tight
    /// performance budget, such as a real time simulation, you may not want to do all of your
    /// coalescing at once.
    ///
    /// This method allows you to split the work over many frames by giving you fine grained control
    /// over which bin sections is getting coalesced and which other bin sections it gets tested
    /// against.
    ///
    /// So, for example, say you have an application where you want to fully coalesce the entire
    /// bin every ten seconds, and you are running at 60 frames per second. You would then
    /// distribute the coalescing work such that it would take 600 calls to compare every bin
    /// section.
    ///
    /// Here's a basic eample of splitting the work.
    ///
    /// ```
    /// # use rectangle_pack::TargetBin;
    /// let target_bin = my_target_bin();
    ///
    /// let current_frame: usize = get_current_frame() % 600;
    ///
    /// for idx in 0..target_bin.available_bin_sections().len() {
    ///     let len = target_bin.available_bin_sections().len();
    ///
    ///     let start = len / 600 * current_frame;
    ///     let end = start + len / 600;
    ///
    ///     target_bin.coalesce_available_sections(idx, start..end);
    /// }
    ///
    /// # fn my_target_bin () -> TargetBin {
    /// #     TargetBin::new(1, 2, 3)
    /// # }
    /// #
    /// # fn get_current_frame () -> usize {
    /// #     0
    /// # }
    /// ```
    ///
    /// [`TargetBin.push_available_bin_section`]: #method.push_available_bin_section
    // TODO: The example of distributing the work will duplicate work at the boundaries between
    //  the splits, so we should fix the example so that it never runs the same calculations twice
    //  regardless of the number of available bins.
    pub fn coalesce_available_sections(bin_section_index: usize, compare_to_indices: Range<usize>) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo() {
        unimplemented!(
            r#"
Write our test cases 
        "#
        )
    }
}
