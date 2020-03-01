use crate::LayeredRect;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// Groups of rectangles that need to be placed into bins.
///
/// When placing groups a heuristic is used to determine which groups are the largest.
/// Larger groups are placed first.
///
/// A group's heuristic is computed by calculating the heuristic of all of the rectangles inside
/// the group and then summing them.
#[derive(Debug)]
pub struct LayeredRectGroups<InboundId: Hash, GroupId: Debug + Hash + Eq> {
    inbound_id_to_group_ids: HashMap<InboundId, Vec<Group<GroupId>>>,
    group_id_to_inbound_ids: HashMap<Group<GroupId>, Vec<InboundId>>,
    rects: HashMap<InboundId, LayeredRect>,
    auto_group_idx: u32,
}

/// A group of rectangles that need to be placed together
#[derive(Debug, Hash, Eq, PartialEq)]
enum Group<GroupId: Debug + Hash + Eq + PartialEq> {
    /// An automatically generated (auto incrementing) group identifier for rectangles that were
    /// passed in without any associated group ids.
    ///
    /// We still want to treat these lone rectangles as their own "groups" so that we can more
    /// easily compare their heuristics against those of other groups.
    ///
    /// If everything is a "group" - comparing groups becomes simpler.
    Ungrouped(u32),
    /// Wraps a user provided group identifier.
    Grouped(GroupId),
}

impl<InboundId: Hash + Clone + Eq, GroupdId: Debug + Hash + Clone + Eq>
    LayeredRectGroups<InboundId, GroupdId>
{
    /// Create a new `LayeredRectGroups`
    pub fn new() -> Self {
        Self {
            inbound_id_to_group_ids: Default::default(),
            group_id_to_inbound_ids: Default::default(),
            rects: Default::default(),
            auto_group_idx: 0,
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
        inbound_id: InboundId,
        group_ids: Option<Vec<GroupdId>>,
        inbound: LayeredRect,
    ) {
        self.rects.insert(inbound_id.clone(), inbound);

        match group_ids {
            None => {
                self.group_id_to_inbound_ids.insert(
                    Group::Ungrouped(self.auto_group_idx),
                    vec![inbound_id.clone()],
                );

                self.inbound_id_to_group_ids
                    .insert(inbound_id, vec![Group::Ungrouped(self.auto_group_idx)]);

                self.auto_group_idx += 1;
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
    use crate::LayeredRect;

    /// Verify that if we insert a rectangle that doesn't have a group it is given an automatic
    /// group ID.
    #[test]
    fn the_first_ungrouped_rectangle_is_assigned_an_automatic_id_of_zero() {
        let mut lrg: LayeredRectGroups<_, ()> = LayeredRectGroups::new();

        lrg.push_rect(InboundId::One, None, LayeredRect::new(10, 10, 1));

        assert_eq!(lrg.auto_group_idx, 1);
        assert_eq!(
            lrg.group_id_to_inbound_ids[&Group::Ungrouped(0)],
            vec![InboundId::One]
        );
    }

    /// Verify that if we insert two rectangles, neither of which are in groups, that are both
    /// given unique auto-generated group IDs.
    #[test]
    fn automatic_ids_auto_increment() {
        let mut lrg: LayeredRectGroups<_, ()> = LayeredRectGroups::new();

        lrg.push_rect(InboundId::One, None, LayeredRect::new(10, 10, 1));
        lrg.push_rect(InboundId::Two, None, LayeredRect::new(10, 10, 1));

        assert_eq!(lrg.auto_group_idx, 2);
        assert_eq!(
            lrg.group_id_to_inbound_ids[&Group::Ungrouped(0)],
            vec![InboundId::One]
        );
        assert_eq!(
            lrg.group_id_to_inbound_ids[&Group::Ungrouped(1)],
            vec![InboundId::Two]
        );
    }

    /// When multiple different rects from the same group are pushed they should be present in the
    /// map of group id -> inbound rect id
    #[test]
    fn group_id_to_inbound_ids() {
        let mut lrg = LayeredRectGroups::new();

        lrg.push_rect(InboundId::One, Some(vec![0]), LayeredRect::new(10, 10, 1));
        lrg.push_rect(InboundId::Two, Some(vec![0]), LayeredRect::new(10, 10, 1));

        assert_eq!(
            lrg.group_id_to_inbound_ids[&Group::Grouped(0)],
            vec![InboundId::One, InboundId::Two]
        );
    }

    /// Verify that we store the map of inbound id -> group ids
    #[test]
    fn inbound_id_to_group_ids() {
        let mut lrg = LayeredRectGroups::new();

        lrg.push_rect(
            InboundId::One,
            Some(vec![0, 1]),
            LayeredRect::new(10, 10, 1),
        );

        lrg.push_rect(InboundId::Two, None, LayeredRect::new(10, 10, 1));

        assert_eq!(
            lrg.inbound_id_to_group_ids[&InboundId::One],
            vec![Group::Grouped(0), Group::Grouped(1)]
        );

        assert_eq!(
            lrg.inbound_id_to_group_ids[&InboundId::Two],
            vec![Group::Ungrouped(0)]
        );
    }

    /// Verify that we store in rectangle associated with its inbound ID
    #[test]
    fn store_the_inbound_rectangle() {
        let mut lrg = LayeredRectGroups::new();

        lrg.push_rect(
            InboundId::One,
            Some(vec![0, 1]),
            LayeredRect::new(10, 10, 1),
        );

        assert_eq!(lrg.rects[&InboundId::One], LayeredRect::new(10, 10, 1));
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum InboundId {
        One,
        Two,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum GroupId {
        Ten,
        Elevent,
    }
}
