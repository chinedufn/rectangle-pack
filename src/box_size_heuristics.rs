use crate::WidthHeightDepth;

/// Incoming boxes are places into the smallest hole that will fit them.
///
/// "small" vs. "large" is based on the heuristic function.
///
/// A larger heuristic means that the box is larger.
pub type BoxSizeHeuristicFn = dyn Fn(WidthHeightDepth) -> u128;

/// The volume of the box
pub fn volume_heuristic(whd: WidthHeightDepth) -> u128 {
    (whd.width * whd.height * whd.depth) as _
}
