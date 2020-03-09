/// Used to represent a volume (or area of the depth is 1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Ord, PartialOrd)]
#[allow(missing_docs)]
pub struct WidthHeightDepth {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth: u32,
}

#[allow(missing_docs)]
impl WidthHeightDepth {
    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        WidthHeightDepth {
            width,
            height,
            depth,
        }
    }
}
