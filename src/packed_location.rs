use crate::width_height_depth::WidthHeightDepth;

/// Describes how and where an incoming rectangle was packed into the target bins
#[derive(Debug, PartialEq)]
pub struct PackedLocation {
    x: u32,
    y: u32,
    z: u32,
    whd: WidthHeightDepth,
    x_axis_rotation: RotatedBy,
    y_axis_rotation: RotatedBy,
    z_axis_rotation: RotatedBy,
}

#[derive(Debug, PartialEq)]
#[allow(unused)] // TODO: Implement rotations
enum RotatedBy {
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
