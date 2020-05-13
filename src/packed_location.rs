use crate::width_height_depth::WidthHeightDepth;

/// Describes how and where an incoming rectangle was packed into the target bins
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PackedLocation {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) z: u32,
    pub(crate) whd: WidthHeightDepth,
    pub(crate) x_axis_rotation: RotatedBy,
    pub(crate) y_axis_rotation: RotatedBy,
    pub(crate) z_axis_rotation: RotatedBy,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(unused)] // TODO: Implement rotations
pub enum RotatedBy {
    ZeroDegrees,
    NinetyDegrees,
}

#[allow(missing_docs)]
impl PackedLocation {
    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn z(&self) -> u32 {
        self.z
    }

    pub fn width(&self) -> u32 {
        self.whd.width
    }

    pub fn height(&self) -> u32 {
        self.whd.height
    }

    pub fn depth(&self) -> u32 {
        self.whd.depth
    }
}
