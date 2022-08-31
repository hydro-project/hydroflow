use std::collections::BTreeMap;

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

    pub fn edge(&self, e: E) -> Option<(V, V)> {
        self.edges.get(e).copied()
    }

    pub fn edges(&self) -> impl '_ + Iterator<Item = (E, (V, V))> {
        self.edges.iter().map(|(e, &(src, dst))| (e, (src, dst)))
    }

    pub fn successors(&self, v: V) -> impl '_ + Iterator<Item = E> {
        self.succs.get(v).into_iter().flatten().copied()
    }

    pub fn predecessors(&self, v: V) -> impl '_ + Iterator<Item = E> {
        self.preds.get(v).into_iter().flatten().copied()
    }

    pub fn degree_out(&self, v: V) -> usize {
        self.succs.get(v).map(Vec::len).unwrap_or_default()
    }

    pub fn degree_in(&self, v: V) -> usize {
        self.preds.get(v).map(Vec::len).unwrap_or_default()
    }
}
