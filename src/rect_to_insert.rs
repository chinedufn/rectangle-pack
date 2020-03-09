use crate::width_height_depth::WidthHeightDepth;

/// A rectangle that we want to insert into a target bin
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RectToInsert {
    pub(crate) whd: WidthHeightDepth,
    allow_global_x_axis_rotation: bool,
    allow_global_y_axis_rotation: bool,
    allow_global_z_axis_rotation: bool,
}

impl Into<WidthHeightDepth> for RectToInsert {
    fn into(self) -> WidthHeightDepth {
        WidthHeightDepth {
            width: self.width(),
            height: self.height(),
            depth: self.depth(),
        }
    }
}

#[allow(missing_docs)]
impl RectToInsert {
    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        RectToInsert {
            whd: WidthHeightDepth {
                width,
                height,
                depth,
            },
            // Rotation is not yet supported
            allow_global_x_axis_rotation: false,
            allow_global_y_axis_rotation: false,
            allow_global_z_axis_rotation: false,
        }
    }
}

#[allow(missing_docs)]
impl RectToInsert {
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
