#![deny(missing_docs)]

use crate::bin_split::BinSection;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

mod bin_split;

fn pack_rects<InboundId: Debug + Hash, Inbound: LayeredRect>(
    incoming: &HashMap<InboundId, Inbound>,
    target_bins: &mut Vec<TargetBin>,
) -> Result<(), RectanglePackError<InboundId>> {
    for (inbound_id, inbound) in incoming.iter() {
        for bin in target_bins.iter_mut() {
            for bin_split in bin.remaining_splits.iter_mut() {
                // TODO: Check if inbound can fit into this bin split - if it can then remove the
                // split, place it into the split and create two new splits and push those to
                // the end of the remaining splits (smallest last)

                // If we can't then move on to the next split
            }
        }

        // If we make it here then no bin was able to fit our inbound rect - return an error
    }

    Ok(())
}

struct RectanglePackOk<InboundId, BinId> {
    locations: HashMap<InboundId, PackedLocation<BinId>>,
}

struct PackedLocation<BinId> {
    bin_id: BinId,
    top_left: [u32; 2],
    bottom_right: [u32; 2],
    // TODO: document the getter
    // x_copy = x
    // x = y
    // y = 1 - x_copy
    is_rotated: bool,
}

#[derive(Debug)]
struct Rect {
    width: u32,
    height: u32,
    layers: u32,
    allow_rotation: bool,
}

impl Rect {
    /// # Panics
    ///
    /// Panics if the layer count is 0 since that would mean we'd be attempting to place nothing.
    pub fn new(width: u32, height: u32, layers: u32, allow_rotation: bool) -> Self {
        assert!(layers > 0);

        Rect {
            width,
            height,
            layers,
            allow_rotation,
        }
    }
}

impl LayeredRect for Rect {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn layers(&self) -> u32 {
        self.layers
    }

    fn allow_rotation(&self) -> bool {
        self.allow_rotation
    }
}

trait LayeredRect {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn layers(&self) -> u32;
    fn allow_rotation(&self) -> bool;
}

#[derive(Debug, thiserror::Error)]
enum RectanglePackError<InboundId: Debug> {
    /// The rectangles can't be placed into the bins. More bin space needs to be provided.
    #[error("The rectangles cannot fit into the bins.")]
    NotEnoughBinSpace { unplaced: Vec<InboundId> },
}

enum ImageId {
    One,
    Two,
}

enum BinId {
    One,
    Two,
}

struct TargetBin {
    max_width: u32,
    max_height: u32,
    layers: u32,
    remaining_splits: Vec<BinSection>,
}

impl TargetBin {
    /// # Panics
    ///
    /// Panics if the layer count is 0 since that would mean we'd be attempting to place rectangles
    /// onto nothing.
    pub fn new(max_width: u32, max_height: u32, layers: u32) -> Self {
        assert!(layers > 0);

        let remaining_splits = vec![BinSection::new(0, 0, max_width, max_height, 0, layers - 1)];

        TargetBin {
            max_width,
            max_height,
            layers,
            remaining_splits,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{pack_rects, Rect, RectanglePackError, TargetBin};
    use std::collections::HashMap;

    /// If the provided rectangles can't fit into the provided bins because one or more rectangles are
    /// too wide we return an error.
    #[test]
    fn error_if_the_rectangles_cannot_fit_due_to_width() {
        let mut incoming = HashMap::new();
        incoming.insert(InboundId::One, Rect::new(3, 1, 1, false));

        let mut target = vec![TargetBin::new(2, 100, 1)];

        match pack_rects(&incoming, &mut target).err().unwrap() {
            RectanglePackError::NotEnoughBinSpace { unplaced } => {
                assert_eq!(unplaced, vec![InboundId::One])
            }
            _ => panic!(),
        };
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum InboundId {
        One,
        Two,
    }
}
