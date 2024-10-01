//! General graph algorithm utility functions

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

/// Computes the topological sort of the nodes of a possibly cyclic graph by ordering strongly
/// connected components together.
pub fn topo_sort_scc<Id, NodesFn, NodeIds, PredsFn, SuccsFn, PredsIter, SuccsIter>(
    mut nodes_fn: NodesFn,
    mut preds_fn: PredsFn,
    succs_fn: SuccsFn,
) -> Vec<Id>
where
    Id: Copy + Eq + Ord,
    NodesFn: FnMut() -> NodeIds,
    NodeIds: IntoIterator<Item = Id>,
    PredsFn: FnMut(Id) -> PredsIter,
    SuccsFn: FnMut(Id) -> SuccsIter,
    PredsIter: IntoIterator<Item = Id>,
    SuccsIter: IntoIterator<Item = Id>,
{
    let scc = scc_kosaraju((nodes_fn)(), &mut preds_fn, succs_fn);
    let topo_sort_order = {
        // Condensed each SCC into a single node for toposort.
        let mut condensed_preds: BTreeMap<Id, Vec<Id>> = Default::default();
        for u in (nodes_fn)() {
            condensed_preds
                .entry(scc[&u])
                .or_default()
                .extend((preds_fn)(u).into_iter().map(|v| scc[&v]));
        }

        topo_sort((nodes_fn)(), |v| {
            condensed_preds.get(&v).into_iter().flatten().cloned()
        })
    };
    topo_sort_order
}

/// Topologically sorts a set of nodes. Returns a list where the order of `Id`s will agree with
/// the order of any path through the graph.
///
/// This naturally requires a directed acyclic graph (DAG).
///
/// <https://en.wikipedia.org/wiki/Topological_sorting>
pub fn topo_sort<Id, NodeIds, PredsFn, PredsIter>(
    node_ids: NodeIds,
    mut preds_fn: PredsFn,
) -> Vec<Id>
where
    Id: Copy + Eq + Ord,
    NodeIds: IntoIterator<Item = Id>,
    PredsFn: FnMut(Id) -> PredsIter,
    PredsIter: IntoIterator<Item = Id>,
{
    let (mut marked, mut order) = Default::default();

    fn pred_dfs_postorder<Id, PredsFn, PredsIter>(
        node_id: Id,
        preds_fn: &mut PredsFn,
        marked: &mut BTreeSet<Id>,
        order: &mut Vec<Id>,
    ) where
        Id: Copy + Eq + Ord,
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

/// Finds the strongly connected components in the graph. A strongly connected component is a
/// subset of nodes that are all reachable by each other.
///
/// <https://en.wikipedia.org/wiki/Strongly_connected_component>
///
/// Each component is represented by a specific member node. The returned `BTreeMap` maps each node
/// ID to the node ID of its "representative." Nodes with the same "representative" node are in the
/// same strongly connected component.
///
/// This function uses [Kosaraju's algorithm](https://en.wikipedia.org/wiki/Kosaraju%27s_algorithm).
pub fn scc_kosaraju<Id, NodeIds, PredsFn, SuccsFn, PredsIter, SuccsIter>(
    nodes: NodeIds,
    mut preds_fn: PredsFn,
    mut succs_fn: SuccsFn,
) -> BTreeMap<Id, Id>
where
    Id: Copy + Eq + Ord,
    NodeIds: IntoIterator<Item = Id>,
    PredsFn: FnMut(Id) -> PredsIter,
    SuccsFn: FnMut(Id) -> SuccsIter,
    PredsIter: IntoIterator<Item = Id>,
    SuccsIter: IntoIterator<Item = Id>,
{
    // https://en.wikipedia.org/wiki/Kosaraju%27s_algorithm
    fn visit<Id, SuccsFn, SuccsIter>(
        succs_fn: &mut SuccsFn,
        u: Id,
        seen: &mut BTreeSet<Id>,
        stack: &mut Vec<Id>,
    ) where
        Id: Copy + Eq + Ord,
        SuccsFn: FnMut(Id) -> SuccsIter,
        SuccsIter: IntoIterator<Item = Id>,
    {
        if seen.insert(u) {
            for v in (succs_fn)(u) {
                visit(succs_fn, v, seen, stack);
            }
            stack.push(u);
        }
    }
    let (mut seen, mut stack) = Default::default();
    for sg in nodes {
        visit(&mut succs_fn, sg, &mut seen, &mut stack);
    }
    let _ = seen;

    fn assign<Id, PredsFn, PredsIter>(
        preds_fn: &mut PredsFn,
        v: Id,
        root: Id,
        components: &mut BTreeMap<Id, Id>,
    ) where
        Id: Copy + Eq + Ord,
        PredsFn: FnMut(Id) -> PredsIter,
        PredsIter: IntoIterator<Item = Id>,
    {
        if let Entry::Vacant(vacant_entry) = components.entry(v) {
            vacant_entry.insert(root);
            for u in (preds_fn)(v) {
                assign(preds_fn, u, root, components);
            }
        }
    }

    let mut components = Default::default();
    for sg in stack.into_iter().rev() {
        assign(&mut preds_fn, sg, sg, &mut components);
    }
    components
}

#[cfg(test)]
mod test {
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

        let position: BTreeMap<_, _> = sort.iter().enumerate().map(|(i, &x)| (x, i)).collect();
        for (src, dst) in edges.iter() {
            assert!(position[src] < position[dst]);
        }
    }

    #[test]
    pub fn test_scc_kosaraju() {
        // https://commons.wikimedia.org/wiki/File:Scc-1.svg
        let edges = [
            ('a', 'b'),
            ('b', 'c'),
            ('b', 'f'),
            ('b', 'e'),
            ('c', 'd'),
            ('c', 'g'),
            ('d', 'c'),
            ('d', 'h'),
            ('e', 'a'),
            ('e', 'f'),
            ('f', 'g'),
            ('g', 'f'),
            ('h', 'd'),
            ('h', 'g'),
        ];

        let scc = scc_kosaraju(
            'a'..='g',
            |v| {
                edges
                    .iter()
                    .filter(move |&&(_, dst)| v == dst)
                    .map(|&(src, _)| src)
            },
            |u| {
                edges
                    .iter()
                    .filter(move |&&(src, _)| u == src)
                    .map(|&(_, dst)| dst)
            },
        );

        assert_ne!(scc[&'a'], scc[&'c']);
        assert_ne!(scc[&'a'], scc[&'f']);
        assert_ne!(scc[&'c'], scc[&'f']);

        assert_eq!(scc[&'a'], scc[&'b']);
        assert_eq!(scc[&'a'], scc[&'e']);

        assert_eq!(scc[&'c'], scc[&'d']);
        assert_eq!(scc[&'c'], scc[&'h']);

        assert_eq!(scc[&'f'], scc[&'g']);
    }
}
