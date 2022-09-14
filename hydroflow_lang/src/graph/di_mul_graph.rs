use std::collections::{BTreeMap, BTreeSet};

use slotmap::{Key, SecondaryMap, SlotMap};

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EdgeIndex(pub usize);

pub type EdgeEnd<V> = (V, EdgeIndex);
pub type AdjEdges<V> = BTreeMap<EdgeIndex, EdgeEnd<V>>;

/// A directed multigraph where an vertex's inbound and outbound edges are indexed.
#[derive(Clone, Debug)]
pub struct DiMulGraph<V, E>
where
    V: Key,
    E: Key,
{
    /// Edges
    pub(crate) edges: SlotMap<E, (V, V)>,

    /// Successors for each vert.
    pub(crate) succs: SecondaryMap<V, Vec<E>>,
    /// Predecessors for each vert.
    pub(crate) preds: SecondaryMap<V, Vec<E>>,
}
impl<V, E> Default for DiMulGraph<V, E>
where
    V: Key,
    E: Key,
{
    fn default() -> Self {
        let (edges, succs, preds) = Default::default();
        Self {
            edges,
            succs,
            preds,
        }
    }
}
impl<V, E> DiMulGraph<V, E>
where
    V: Key,
    E: Key,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            // Estimate 1 edge per vertex.
            edges: SlotMap::with_capacity_and_key(capacity),
            succs: SecondaryMap::with_capacity(capacity),
            preds: SecondaryMap::with_capacity(capacity),
        }
    }

    /// Assert that `self` is in a consistent state, for debugging.
    /// This is computationally expensive for large graphs.
    pub fn assert_valid(&self) {
        // Ensure each edge exists in the adj lists.
        for (edge_id, &(src, dst)) in self.edges.iter() {
            assert!(self.succs[src].contains(&edge_id));
            assert!(self.preds[dst].contains(&edge_id));
        }

        // Ensure no duplicate preds or succs
        for succ_list in self.succs.values() {
            let set: BTreeSet<&E> = succ_list.iter().collect();
            assert_eq!(set.len(), succ_list.len());
        }
        for pred_list in self.succs.values() {
            let set: BTreeSet<&E> = pred_list.iter().collect();
            assert_eq!(set.len(), pred_list.len());
        }

        // Missing edges and duplicate edges could cancel each other out. But
        // this would be caught by the above.
        assert_eq!(
            self.edges.len(),
            self.succs.values().map(|vec| vec.len()).sum::<usize>(),
            "succs broken (contains duplicate or removed edge?)"
        );
        assert_eq!(
            self.edges.len(),
            self.preds.values().map(|vec| vec.len()).sum::<usize>(),
            "preds broken (contains duplicate or removed edge?)"
        );
    }

    fn get_adj_edges(adj_list: &mut SecondaryMap<V, Vec<E>>, v: V) -> &mut Vec<E> {
        if !adj_list.contains_key(v) {
            adj_list.insert(v, Default::default());
        }
        &mut adj_list[v]
    }

    pub fn insert_edge(&mut self, src: V, dst: V) -> E {
        let e = self.edges.insert((src, dst));
        Self::get_adj_edges(&mut self.succs, src).push(e);
        Self::get_adj_edges(&mut self.preds, dst).push(e);
        e
    }

    /// For an edge E from A -> B, insert a new node NODE along that edge to
    /// create A -> NODE -> B. Returns the edge ID into and out of NODE
    /// respectively.
    ///
    /// Returns None if the edge doesn't exist.
    pub fn insert_intermediate_node(&mut self, new_node: V, edge: E) -> Option<(E, E)> {
        self.assert_valid();

        // Remove old edge from edges.
        let (src, dst) = self.edges.remove(edge)?;

        // Insert new edges into edges.
        let e0 = self.edges.insert((src, new_node));
        let e1 = self.edges.insert((new_node, dst));

        // Remove old & add new edges in succs/preds.
        let succ_vec_idx = self.succs[src].iter().position(|&e| e == edge).unwrap();
        let pred_vec_idx = self.preds[dst].iter().position(|&e| e == edge).unwrap();
        assert_eq!(
            edge,
            std::mem::replace(&mut self.succs[src][succ_vec_idx], e0)
        );
        assert_eq!(
            edge,
            std::mem::replace(&mut self.preds[dst][pred_vec_idx], e1)
        );

        // Insert new node succs/preds.
        assert!(
            self.preds.insert(new_node, vec![e0]).is_none(),
            "Cannot insert intermediate node that already exists"
        );
        assert!(
            self.succs.insert(new_node, vec![e1]).is_none(),
            "Cannot insert intermediate node that already exists"
        );

        self.assert_valid();
        Some((e0, e1))
    }

    pub fn edge(&self, e: E) -> Option<(V, V)> {
        self.edges.get(e).copied()
    }

    pub fn edges(&self) -> impl '_ + Iterator<Item = (E, (V, V))> {
        self.edges.iter().map(|(e, &(src, dst))| (e, (src, dst)))
    }

    pub fn successor_edges(&self, v: V) -> impl '_ + Iterator<Item = E> {
        self.succs.get(v).into_iter().flatten().copied()
    }

    pub fn predecessor_edges(&self, v: V) -> impl '_ + Iterator<Item = E> {
        self.preds.get(v).into_iter().flatten().copied()
    }

    pub fn successors(&self, v: V) -> impl '_ + Iterator<Item = V> {
        self.successor_edges(v)
            .map(|edge_id| self.edges[edge_id])
            .map(|(_v, succ)| succ)
    }

    pub fn predecessors(&self, v: V) -> impl '_ + Iterator<Item = V> {
        self.predecessor_edges(v)
            .map(|edge_id| self.edges[edge_id])
            .map(|(pred, _v)| pred)
    }

    pub fn degree_out(&self, v: V) -> usize {
        self.succs.get(v).map(Vec::len).unwrap_or_default()
    }

    pub fn degree_in(&self, v: V) -> usize {
        self.preds.get(v).map(Vec::len).unwrap_or_default()
    }
}
