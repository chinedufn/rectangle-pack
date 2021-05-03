use crate::RectToInsert;

#[cfg(not(std))]
use alloc::collections::BTreeMap as KeyValMap;
#[cfg(std)]
use std::collections::HashMap as KeyValMap;

use alloc::{
    collections::{btree_map::Entry, BTreeMap},
    vec::Vec,
};
use core::{fmt::Debug, hash::Hash};

/// Groups of rectangles that need to be placed into bins.
///
/// When placing groups a heuristic is used to determine which groups are the largest.
/// Larger groups are placed first.
///
/// A group's heuristic is computed by calculating the heuristic of all of the rectangles inside
/// the group and then summing them.
#[derive(Debug)]
pub struct GroupedRectsToPlace<RectToPlaceId, GroupId = ()>
where
    RectToPlaceId: Debug + Hash + Eq + Ord + PartialOrd,
    GroupId: Debug + Hash + Eq + Ord + PartialOrd,
{
    // FIXME: inbound_id_to_group_id appears to be unused. If so, remove it. Also remove the
    //  Hash and Eq constraints on RectToPlaceId if we remove this map
    pub(crate) inbound_id_to_group_ids:
        KeyValMap<RectToPlaceId, Vec<Group<GroupId, RectToPlaceId>>>,
    pub(crate) group_id_to_inbound_ids: BTreeMap<Group<GroupId, RectToPlaceId>, Vec<RectToPlaceId>>,
    pub(crate) rects: KeyValMap<RectToPlaceId, RectToInsert>,
}

/// A group of rectangles that need to be placed together
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Group<GroupId, RectToPlaceId>
where
    GroupId: Debug + Hash + Eq + PartialEq + Ord + PartialOrd,
    RectToPlaceId: Debug + Ord + PartialOrd,
{
    /// An automatically generated (auto incrementing) group identifier for rectangles that were
    /// passed in without any associated group ids.
    ///
    /// We still want to treat these lone rectangles as their own "groups" so that we can more
    /// easily compare their heuristics against those of other groups.
    ///
    /// If everything is a "group" - comparing groups becomes simpler.
    Ungrouped(RectToPlaceId),
    /// Wraps a user provided group identifier.
    Grouped(GroupId),
}

impl<RectToPlaceId, GroupId> GroupedRectsToPlace<RectToPlaceId, GroupId>
where
    RectToPlaceId: Debug + Hash + Clone + Eq + Ord + PartialOrd,
    GroupId: Debug + Hash + Clone + Eq + Ord + PartialOrd,
{
    /// Create a new `LayeredRectGroups`
    pub fn new() -> Self {
        Self {
            inbound_id_to_group_ids: Default::default(),
            group_id_to_inbound_ids: Default::default(),
            rects: Default::default(),
        }
    }

    /// Push one or more rectangles
    ///
    /// # Panics
    ///
    /// Panics if a `Some(Vec<GroupId>)` passed in but the length is 0, as this is likely a
    /// mistake and `None` should be used instead.
    pub fn push_rect(
        &mut self,
        inbound_id: RectToPlaceId,
        group_ids: Option<Vec<GroupId>>,
        inbound: RectToInsert,
    ) {
        self.rects.insert(inbound_id.clone(), inbound);

        match group_ids {
            None => {
                self.group_id_to_inbound_ids.insert(
                    Group::Ungrouped(inbound_id.clone()),
                    vec![inbound_id.clone()],
                );

                self.inbound_id_to_group_ids
                    .insert(inbound_id.clone(), vec![Group::Ungrouped(inbound_id)]);
            }
            Some(group_ids) => {
                self.inbound_id_to_group_ids.insert(
                    inbound_id.clone(),
                    group_ids
                        .clone()
                        .into_iter()
                        .map(|gid| Group::Grouped(gid))
                        .collect(),
                );

                for group_id in group_ids {
                    match self.group_id_to_inbound_ids.entry(Group::Grouped(group_id)) {
                        Entry::Occupied(mut o) => {
                            o.get_mut().push(inbound_id.clone());
                        }
                        Entry::Vacant(v) => {
                            v.insert(vec![inbound_id.clone()]);
                        }
                    };
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RectToInsert;

    /// Verify that if we insert a rectangle that doesn't have a group it is given a group ID based
    /// on its RectToPlaceId.
    #[test]
    fn ungrouped_rectangles_use_their_inbound_id_as_their_group_id() {
        let mut lrg: GroupedRectsToPlace<_, ()> = GroupedRectsToPlace::new();

        lrg.push_rect(RectToPlaceId::One, None, RectToInsert::new(10, 10, 1));

        assert_eq!(
            lrg.group_id_to_inbound_ids[&Group::Ungrouped(RectToPlaceId::One)],
            vec![RectToPlaceId::One]
        );
    }

    /// When multiple different rects from the same group are pushed they should be present in the
    /// map of group id -> inbound rect id
    #[test]
    fn group_id_to_inbound_ids() {
        let mut lrg = GroupedRectsToPlace::new();

        lrg.push_rect(
            RectToPlaceId::One,
            Some(vec![0]),
            RectToInsert::new(10, 10, 1),
        );
        lrg.push_rect(
            RectToPlaceId::Two,
            Some(vec![0]),
            RectToInsert::new(10, 10, 1),
        );

        assert_eq!(
            lrg.group_id_to_inbound_ids.get(&Group::Grouped(0)).unwrap(),
            &vec![RectToPlaceId::One, RectToPlaceId::Two]
        );
    }

    /// Verify that we store the map of inbound id -> group ids
    #[test]
    fn inbound_id_to_group_ids() {
        let mut lrg = GroupedRectsToPlace::new();

        lrg.push_rect(
            RectToPlaceId::One,
            Some(vec![0, 1]),
            RectToInsert::new(10, 10, 1),
        );

        lrg.push_rect(RectToPlaceId::Two, None, RectToInsert::new(10, 10, 1));

        assert_eq!(
            lrg.inbound_id_to_group_ids[&RectToPlaceId::One],
            vec![Group::Grouped(0), Group::Grouped(1)]
        );

        assert_eq!(
            lrg.inbound_id_to_group_ids[&RectToPlaceId::Two],
            vec![Group::Ungrouped(RectToPlaceId::Two)]
        );
    }

    /// Verify that we store in rectangle associated with its inbound ID
    #[test]
    fn store_the_inbound_rectangle() {
        let mut lrg = GroupedRectsToPlace::new();

        lrg.push_rect(
            RectToPlaceId::One,
            Some(vec![0, 1]),
            RectToInsert::new(10, 10, 1),
        );

        assert_eq!(lrg.rects[&RectToPlaceId::One], RectToInsert::new(10, 10, 1));
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
    enum RectToPlaceId {
        One,
        Two,
    }
}
