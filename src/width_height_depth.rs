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
    /// # Panics
    ///
    /// Panics if width, height or depth is 0.
    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        assert_ne!(width, 0);
        assert_ne!(height, 0);
        assert_ne!(depth, 0);

        WidthHeightDepth {
            width,
            height,
            depth,
        }
    }

    pub fn volume(&self) -> u128 {
        self.width as u128 * self.height as u128 * self.depth as u128
    }
}
