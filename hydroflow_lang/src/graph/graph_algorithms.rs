use std::collections::HashSet;
use std::hash::Hash;

pub fn topo_sort<Id, NodeIds, PredsFn, PredsIter>(
    node_ids: NodeIds,
    mut preds_fn: PredsFn,
) -> Vec<Id>
where
    Id: Copy + Hash + Eq,
    NodeIds: IntoIterator<Item = Id>,
    PredsFn: FnMut(Id) -> PredsIter,
    PredsIter: IntoIterator<Item = Id>,
{
    let (mut marked, mut order) = Default::default();

    fn pred_dfs_postorder<Id, PredsFn, PredsIter>(
        node_id: Id,
        preds_fn: &mut PredsFn,
        marked: &mut HashSet<Id>,
        order: &mut Vec<Id>,
    ) where
        Id: Copy + Hash + Eq,
        PredsFn: FnMut(Id) -> PredsIter,
        PredsIter: IntoIterator<Item = Id>,
    {
        if marked.insert(node_id) {
            for next_pred in (preds_fn)(node_id) {
                pred_dfs_postorder(next_pred, preds_fn, marked, order);
            }
            order.push(node_id);
        }
    }

    for node_id in node_ids {
        pred_dfs_postorder(node_id, &mut preds_fn, &mut marked, &mut order);
    }

    order
}
