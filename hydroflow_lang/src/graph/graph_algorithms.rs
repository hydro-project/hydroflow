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
        } else {
            // TODO(mingwei): cycle found!
        }
    }

    for node_id in node_ids {
        pred_dfs_postorder(node_id, &mut preds_fn, &mut marked, &mut order);
    }

    order
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    #[test]
    pub fn test_toposort() {
        let edges = [
            (5, 11),
            (11, 2),
            (11, 9),
            (11, 10),
            (7, 11),
            (7, 8),
            (8, 9),
            (3, 8),
            (3, 10),
        ];

        // https://commons.wikimedia.org/wiki/File:Directed_acyclic_graph_2.svg
        let sort = topo_sort([2, 3, 5, 7, 8, 9, 10, 11], |v| {
            edges
                .iter()
                .filter(move |&&(_, dst)| v == dst)
                .map(|&(src, _)| src)
        });
        println!("{:?}", sort);

        let position: HashMap<_, _> = sort.iter().enumerate().map(|(i, &x)| (x, i)).collect();
        for (src, dst) in edges.iter() {
            assert!(position[src] < position[dst]);
        }
    }
}
