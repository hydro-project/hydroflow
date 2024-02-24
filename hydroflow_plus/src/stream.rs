use std::cell::RefCell;
use std::hash::Hash;
use std::io;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use hydroflow::bytes::{Bytes, BytesMut};
use hydroflow::futures::Sink;
use hydroflow_lang::parse::Pipeline;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::de::DeserializeOwned;
use serde::Serialize;
use stageleft::{q, IntoQuotedMut, Quoted};
use syn::parse_quote;
use syn::visit_mut::VisitMut;

use crate::ir::{HfPlusNode, HfPlusSource};
use crate::location::{
    Cluster, HfSendManyToMany, HfSendManyToOne, HfSendOneToMany, HfSendOneToOne, Location,
};

/// Marks the stream as being asynchronous, which means the presence
/// of all elements is directly influenced by the runtime's batching
/// behavior. Aggregation operations are not permitted on streams
/// with this tag because the developer has not explicitly specified
/// if they want to aggregate over the entire stream or just the
/// current batch.
pub struct Async {}

/// Marks the stream as being windowed, which means the developer has
/// opted-into either a batched or persistent windowing semantics.
/// Aggregation operations are permitted on streams with this tag.
pub struct Windowed {}

/// An infinite stream of elements of type `T`.
///
/// Type Parameters:
/// - `'a`: the lifetime of the final Hydroflow graph, which constraints
///   which values can be captured in closures passed to operators
/// - `T`: the type of elements in the stream
/// - `W`: the windowing semantics of the stream, which is either [`Async`]
///    or [`Windowed`]
/// - `N`: the type of the node that the stream is materialized on
pub struct Stream<'a, T, W, N: Location<'a>> {
    pub(crate) node: N,

    pub(crate) ir_leaves: &'a RefCell<Vec<HfPlusNode>>,
    pub(crate) ir_node: RefCell<HfPlusNode>,

    /// Does this stream represent a delta of the underlying data?
    /// i.e. is the true stream supposed to have a `persist()` at the end
    pub(crate) is_delta: bool,
    pub(crate) _phantom: PhantomData<(&'a mut &'a (), T, W)>,
}

impl<'a, T: Clone, W, N: Location<'a>> Clone for Stream<'a, T, W, N> {
    fn clone(&self) -> Self {
        if !matches!(self.ir_node.borrow().deref(), HfPlusNode::Tee { .. }) {
            let orig_ir_node = self.ir_node.replace(HfPlusNode::Todo);
            *self.ir_node.borrow_mut() = HfPlusNode::Tee {
                inner: Rc::new(RefCell::new(orig_ir_node)),
            };
        }

        if let HfPlusNode::Tee { inner } = self.ir_node.borrow().deref() {
            Stream {
                node: self.node.clone(),
                ir_leaves: self.ir_leaves,
                ir_node: RefCell::new(HfPlusNode::Tee {
                    inner: inner.clone(),
                }),
                is_delta: self.is_delta,
                _phantom: PhantomData,
            }
        } else {
            unreachable!()
        }
    }
}

impl<'a, T, W, N: Location<'a>> Stream<'a, T, W, N> {
    fn pipeline_op<U, W2>(self, pipeline: Pipeline, produces_delta: bool) -> Stream<'a, U, W2, N> {
        Stream {
            node: self.node.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(HfPlusNode::PipelineOp {
                input: Box::new(self.ir_node.into_inner()),
                pipeline,
                produces_delta,
            }),
            is_delta: produces_delta,
            _phantom: PhantomData,
        }
    }

    /// Reifies the stream to ensure that it is not producing deltas.
    pub(crate) fn ensure_concrete(self) -> Stream<'a, T, Windowed, N> {
        if self.is_delta {
            self.pipeline_op(parse_quote!(persist()), false)
        } else {
            self.assume_windowed()
        }
    }

    pub fn map<U, F: Fn(T) -> U + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Stream<'a, U, W, N> {
        let f = f.splice();
        let is_delta = self.is_delta;
        self.pipeline_op(parse_quote!(map(#f)), is_delta)
    }

    pub fn flat_map<U, I: IntoIterator<Item = U>, F: Fn(T) -> I + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, N> {
        let f = f.splice();
        let is_delta = self.is_delta;
        self.pipeline_op(parse_quote!(flat_map(#f)), is_delta)
    }

    pub fn enumerate(self) -> Stream<'a, (usize, T), W, N> {
        self.ensure_concrete()
            .pipeline_op(parse_quote!(enumerate()), false)
    }

    pub fn inspect<F: Fn(&T) + 'a>(self, f: impl IntoQuotedMut<'a, F>) -> Stream<'a, T, W, N> {
        let f = f.splice();
        self.ensure_concrete()
            .pipeline_op(parse_quote!(inspect(#f)), false)
    }

    pub fn filter<F: Fn(&T) -> bool + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, T, W, N> {
        let f = f.splice();
        let is_delta = self.is_delta;
        self.pipeline_op(parse_quote!(filter(#f)), is_delta)
    }

    pub fn filter_map<U, F: Fn(T) -> Option<U> + 'a>(
        self,
        f: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, U, W, N> {
        let f = f.splice();
        let is_delta = self.is_delta;
        self.pipeline_op(parse_quote!(filter_map(#f)), is_delta)
    }

    // TODO(shadaj): should allow for differing windows, using strongest one
    pub fn cross_product<O>(self, other: Stream<'a, O, W, N>) -> Stream<'a, (T, O), W, N> {
        if self.node.id() != other.node.id() {
            panic!("cross_product must be called on streams on the same node");
        }

        let output_delta = match (self.is_delta, other.is_delta) {
            (true, true) => {
                false // TODO(shadaj): cross_join already replays?
            }
            (true, false) => false,
            (false, true) => false,
            (false, false) => false,
        };

        Stream {
            node: self.node.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(HfPlusNode::CrossProduct(
                Box::new(self.ir_node.into_inner()),
                Box::new(other.ir_node.into_inner()),
            )),
            is_delta: output_delta,
            _phantom: PhantomData,
        }
    }

    pub fn union(self, other: Stream<'a, T, W, N>) -> Stream<'a, T, W, N> {
        if self.node.id() != other.node.id() {
            panic!("union must be called on streams on the same node");
        }

        let self_concrete = self.ensure_concrete();
        let other_concrete = other.ensure_concrete();

        Stream {
            node: self_concrete.node.clone(),
            ir_leaves: self_concrete.ir_leaves,
            ir_node: RefCell::new(HfPlusNode::Union(
                Box::new(self_concrete.ir_node.into_inner()),
                Box::new(other_concrete.ir_node.into_inner()),
            )),
            is_delta: false,
            _phantom: PhantomData,
        }
    }

    pub fn for_each<F: Fn(T) + 'a>(self, f: impl IntoQuotedMut<'a, F>) {
        let self_concrete = self.ensure_concrete();
        let f = f.splice();

        self_concrete
            .ir_leaves
            .borrow_mut()
            .push(HfPlusNode::ForEach {
                input: Box::new(self_concrete.ir_node.into_inner()),
                f: syn::parse2::<syn::Expr>(f).unwrap().into(),
            });
    }

    pub fn dest_sink<S: Unpin + Sink<T> + 'a>(self, sink: impl Quoted<'a, S>) {
        let self_concrete = self.ensure_concrete();
        let sink = sink.splice();

        self_concrete
            .ir_leaves
            .borrow_mut()
            .push(HfPlusNode::DestSink {
                sink: syn::parse2::<syn::Expr>(sink).unwrap().into(),
                input: Box::new(self_concrete.ir_node.into_inner()),
                send_delta: false,
            });
    }

    pub fn all_ticks(self) -> Stream<'a, T, Windowed, N> {
        if self.is_delta {
            self.ensure_concrete().all_ticks()
        } else {
            Stream {
                node: self.node.clone(),
                ir_leaves: self.ir_leaves,
                ir_node: RefCell::new(HfPlusNode::Persist(Box::new(self.ir_node.into_inner()))),
                is_delta: true,
                _phantom: PhantomData,
            }
        }
    }

    pub fn assume_windowed(self) -> Stream<'a, T, Windowed, N> {
        Stream {
            node: self.node.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: self.ir_node,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, N: Location<'a>> Stream<'a, T, Async, N> {
    pub fn tick_batch(self) -> Stream<'a, T, Windowed, N> {
        Stream {
            node: self.node.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: self.ir_node,
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, N: Location<'a>> Stream<'a, T, Windowed, N> {
    pub fn fold<A, I: Fn() -> A + 'a, C: Fn(&mut A, T)>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, A, Windowed, N> {
        let init = init.splice();
        let comb = comb.splice();

        if self.is_delta {
            self.pipeline_op(parse_quote!(fold::<'static>(#init, #comb)), false)
        } else {
            self.pipeline_op(parse_quote!(fold::<'tick>(#init, #comb)), false)
        }
    }

    pub fn reduce<C: Fn(&mut T, T) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, T, Windowed, N> {
        let comb = comb.splice();

        if self.is_delta {
            self.pipeline_op(parse_quote!(reduce::<'static>(#comb)), false)
        } else {
            self.pipeline_op(parse_quote!(reduce::<'tick>(#comb)), false)
        }
    }

    pub fn count(self) -> Stream<'a, usize, Windowed, N> {
        self.fold(q!(|| 0usize), q!(|count, _| *count += 1))
    }

    pub fn delta(self) -> Stream<'a, T, Windowed, N> {
        if self.is_delta {
            Stream {
                node: self.node.clone(),
                ir_leaves: self.ir_leaves,
                ir_node: self.ir_node,
                is_delta: false,
                _phantom: PhantomData,
            }
        } else {
            self.pipeline_op(parse_quote!(multiset_delta()), false)
        }
    }

    pub fn unique(self) -> Stream<'a, T, Windowed, N>
    where
        T: Eq + Hash,
    {
        self.ensure_concrete()
            .pipeline_op(parse_quote!(unique::<'tick>()), false)
    }

    pub fn filter_not_in(self, other: Stream<'a, T, Windowed, N>) -> Stream<'a, T, Windowed, N>
    where
        T: Eq + Hash,
    {
        if self.node.id() != other.node.id() {
            panic!("union must be called on streams on the same node");
        }

        let self_node = self.node.clone();
        let self_is_delta = self.is_delta;
        let self_ir_leaves = self.ir_leaves;
        let self_ir_node = self.ir_node.borrow().clone();

        let output_delta = match (self_is_delta, other.is_delta) {
            (true, true) => false,
            (true, false) => false,
            (false, true) => false,
            (false, false) => false,
        };

        Stream {
            node: self_node,
            ir_leaves: self_ir_leaves,
            ir_node: RefCell::new(HfPlusNode::Difference(
                Box::new(self_ir_node),
                Box::new(other.ir_node.into_inner()),
            )),
            is_delta: output_delta,
            _phantom: PhantomData,
        }
    }

    pub fn sample_every(
        self,
        duration: impl Quoted<'a, std::time::Duration> + Copy + 'a,
    ) -> Stream<'a, T, Windowed, N> {
        let samples = self.node.source_interval(duration).tick_batch();
        self.cross_product(samples).map(q!(|(a, _)| a))
    }
}

impl<'a, T: Clone, W, N: Location<'a>> Stream<'a, &T, W, N> {
    pub fn cloned(self) -> Stream<'a, T, W, N> {
        let self_is_delta = self.is_delta;
        self.pipeline_op(parse_quote!(map(|d| d.clone())), self_is_delta)
    }
}

impl<'a, K, V1, W, N: Location<'a>> Stream<'a, (K, V1), W, N> {
    // TODO(shadaj): figure out window semantics
    pub fn join<W2, V2>(self, n: Stream<'a, (K, V2), W2, N>) -> Stream<'a, (K, (V1, V2)), W, N>
    where
        K: Eq + Hash,
    {
        if self.node.id() != n.node.id() {
            panic!("join must be called on streams on the same node");
        }

        let output_delta = match (self.is_delta, n.is_delta) {
            (true, true) => {
                false // TODO(shadaj): join already replays?
            }
            (true, false) => false,
            (false, true) => false,
            (false, false) => false,
        };

        Stream {
            node: self.node.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(HfPlusNode::Join(
                Box::new(self.ir_node.into_inner()),
                Box::new(n.ir_node.into_inner()),
            )),
            is_delta: output_delta,
            _phantom: PhantomData,
        }
    }

    pub fn anti_join<W2>(self, n: Stream<'a, K, W2, N>) -> Stream<'a, (K, V1), W, N>
    where
        K: Eq + Hash,
    {
        if self.node.id() != n.node.id() {
            panic!("anti_join must be called on streams on the same node");
        }

        let self_is_delta = self.is_delta;
        let node = self.node.clone();
        let ir_leaves = self.ir_leaves;
        let ir_node = self.ir_node.borrow().clone();

        let output_delta = match (self_is_delta, n.is_delta) {
            (true, true) => false,
            (true, false) => false,
            (false, true) => false,
            (false, false) => false,
        };

        Stream {
            node,
            ir_leaves,
            ir_node: RefCell::new(HfPlusNode::AntiJoin(
                Box::new(ir_node),
                Box::new(n.ir_node.into_inner()),
            )),
            is_delta: output_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, K: Eq + Hash, V, N: Location<'a>> Stream<'a, (K, V), Windowed, N> {
    pub fn fold_keyed<A, I: Fn() -> A + 'a, C: Fn(&mut A, V) + 'a>(
        self,
        init: impl IntoQuotedMut<'a, I>,
        comb: impl IntoQuotedMut<'a, C>,
    ) -> Stream<'a, (K, A), Windowed, N> {
        let init = init.splice();
        let comb = comb.splice();

        if self.is_delta {
            self.pipeline_op(parse_quote!(fold_keyed::<'static>(#init, #comb)), false)
        } else {
            self.pipeline_op(parse_quote!(fold_keyed::<'tick>(#init, #comb)), false)
        }
    }

    pub fn reduce_keyed<F: Fn(&mut V, V) + 'a>(
        self,
        comb: impl IntoQuotedMut<'a, F>,
    ) -> Stream<'a, (K, V), Windowed, N> {
        let comb = comb.splice();

        if self.is_delta {
            self.pipeline_op(parse_quote!(reduce_keyed::<'static>(#comb)), false)
        } else {
            self.pipeline_op(parse_quote!(reduce_keyed::<'tick>(#comb)), false)
        }
    }
}

fn get_this_crate() -> TokenStream {
    let hydroflow_crate = proc_macro_crate::crate_name("hydroflow_plus")
        .expect("hydroflow_plus should be present in `Cargo.toml`");
    match hydroflow_crate {
        proc_macro_crate::FoundCrate::Itself => quote! { hydroflow_plus },
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            quote! { #ident }
        }
    }
}

// TODO(shadaj): has to be public due to temporary stageleft limitations
/// Rewrites use of alloc::string::* to use std::string::*
pub struct RewriteAlloc {}
impl VisitMut for RewriteAlloc {
    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        if i.segments.iter().take(2).collect::<Vec<_>>()
            == vec![
                &syn::PathSegment::from(syn::Ident::new("alloc", Span::call_site())),
                &syn::PathSegment::from(syn::Ident::new("string", Span::call_site())),
            ]
        {
            *i.segments.first_mut().unwrap() =
                syn::PathSegment::from(syn::Ident::new("std", Span::call_site()));
        }
    }
}

fn node_send_direct<'a, T, W, N: Location<'a>>(me: &Stream<'a, T, W, N>, sink: TokenStream) {
    me.ir_leaves.borrow_mut().push(HfPlusNode::DestSink {
        input: Box::new(me.ir_node.replace(HfPlusNode::Todo)),
        sink: syn::parse2::<syn::Expr>(sink).unwrap().into(),
        send_delta: me.is_delta,
    });
}

fn node_send_bincode<'a, T: Serialize, W, N: Location<'a>>(
    me: &Stream<'a, T, W, N>,
    sink: TokenStream,
) {
    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let mut t_type: syn::Type = syn::parse_str(std::any::type_name::<T>()).unwrap();
    RewriteAlloc {}.visit_type_mut(&mut t_type);

    me.ir_leaves.borrow_mut().push(HfPlusNode::DestSink {
        input: Box::new(HfPlusNode::PipelineOp {
            input: Box::new(me.ir_node.replace(HfPlusNode::Todo)),
            pipeline: parse_quote! {
                map(|data| {
                    #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into()
                })
            },
            produces_delta: me.is_delta,
        }),
        send_delta: me.is_delta,
        sink: syn::parse2::<syn::Expr>(sink).unwrap().into(),
    });
}

fn cluster_demux_bincode<'a, T, W, N: Location<'a>>(
    me: &Stream<'a, (u32, T), W, N>,
    sink: TokenStream,
) {
    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let mut t_type: syn::Type = syn::parse_str(std::any::type_name::<T>()).unwrap();
    RewriteAlloc {}.visit_type_mut(&mut t_type);

    me.ir_leaves
        .borrow_mut()
        .push(HfPlusNode::DestSink {
            input: Box::new(HfPlusNode::PipelineOp {
                input: Box::new(me.ir_node.replace(HfPlusNode::Todo)),
                pipeline: parse_quote! {
                    map(|(id, data)| {
                        (id, #root::runtime_support::bincode::serialize::<#t_type>(&data).unwrap().into())
                    })
                },
                produces_delta: me.is_delta,
            }),
            sink: syn::parse2::<syn::Expr>(sink).unwrap().into(),
            send_delta: me.is_delta,
        });
}

fn node_recv_direct<'a, T, W, N: Location<'a>, N2: Location<'a>>(
    me: &Stream<'a, T, W, N>,
    other: &N2,
    source: TokenStream,
) -> HfPlusNode {
    HfPlusNode::Source {
        source: HfPlusSource::Stream(syn::parse2::<syn::Expr>(source).unwrap().into()),
        location_id: other.id(),
        produces_delta: me.is_delta,
    }
}

fn node_recv_bincode<'a, T1, T2: DeserializeOwned, W, N: Location<'a>, N2: Location<'a>>(
    me: &Stream<'a, T1, W, N>,
    other: &N2,
    source: TokenStream,
    tagged: bool,
) -> HfPlusNode {
    let root = get_this_crate();

    // This may fail when instantiated in an environment with different deps
    let mut t_type: syn::Type = syn::parse_str(std::any::type_name::<T2>()).unwrap();
    RewriteAlloc {}.visit_type_mut(&mut t_type);

    if tagged {
        HfPlusNode::PipelineOp {
            pipeline: parse_quote! {
                map(|res| {
                    let (id, b) = res.unwrap();
                    (id, #root::runtime_support::bincode::deserialize::<#t_type>(&b).unwrap())
                })
            },
            input: Box::new(HfPlusNode::Source {
                source: HfPlusSource::Stream(syn::parse2::<syn::Expr>(source).unwrap().into()),
                location_id: other.id(),
                produces_delta: me.is_delta,
            }),
            produces_delta: me.is_delta,
        }
    } else {
        HfPlusNode::PipelineOp {
            pipeline: parse_quote! {
                map(|res| {
                    #root::runtime_support::bincode::deserialize::<#t_type>(&res.unwrap()).unwrap()
                })
            },
            input: Box::new(HfPlusNode::Source {
                source: HfPlusSource::Stream(syn::parse2::<syn::Expr>(source).unwrap().into()),
                location_id: other.id(),
                produces_delta: me.is_delta,
            }),
            produces_delta: me.is_delta,
        }
    }
}

impl<'a, W, N: Location<'a>> Stream<'a, Bytes, W, N> {
    pub fn send_bytes<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(&self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let recv_ir = node_recv_direct(&self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            node: other.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(recv_ir),
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn send_bytes_tagged<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(&self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let recv_ir = node_recv_direct(&self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            node: other.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(recv_ir),
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn send_bytes_interleaved<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        self.send_bytes_tagged(other).map(q!(|r| r.map(|(_, b)| b)))
    }

    pub fn broadcast_bytes<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned().all_ticks();
        other_ids
            .cross_product(self.assume_windowed())
            .demux_bytes(other)
    }

    pub fn broadcast_bytes_tagged<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned().all_ticks();
        other_ids
            .cross_product(self.assume_windowed())
            .demux_bytes_tagged(other)
    }

    pub fn broadcast_bytes_interleaved<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        self.broadcast_bytes_tagged(other)
            .map(q!(|r| r.map(|(_, b)| b)))
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: Location<'a>> Stream<'a, T, W, N> {
    pub fn send_bincode<N2: Location<'a>>(self, other: &N2) -> Stream<'a, T, Async, N2>
    where
        N: HfSendOneToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_bincode(&self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let recv_ir = node_recv_bincode::<_, T, _, _, _>(
            &self,
            other,
            N::gen_source_statement(other, &recv_port),
            false,
        );

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            node: other.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(recv_ir),
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn send_bincode_tagged<N2: Location<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_bincode(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let recv_ir = node_recv_bincode::<_, T, _, _, _>(
            self,
            other,
            N::gen_source_statement(other, &recv_port),
            true,
        );

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            node: other.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(recv_ir),
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn send_bincode_interleaved<N2: Location<'a>>(&self, other: &N2) -> Stream<'a, T, Async, N2>
    where
        N: HfSendManyToOne<'a, N2>,
    {
        self.send_bincode_tagged(other).map(q!(|(_, b)| b))
    }

    pub fn broadcast_bincode<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, T, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned().all_ticks();
        other_ids
            .cross_product(self.assume_windowed())
            .demux_bincode(other)
    }

    pub fn broadcast_bincode_tagged<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let other_ids = self.node.source_iter(other.ids()).cloned().all_ticks();
        other_ids
            .cross_product(self.assume_windowed())
            .demux_bincode_tagged(other)
    }

    pub fn broadcast_bincode_interleaved<N2: Location<'a> + Cluster<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, T, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        self.broadcast_bincode_tagged(other).map(q!(|(_, b)| b))
    }
}

impl<'a, W, N: Location<'a>> Stream<'a, (u32, Bytes), W, N> {
    pub fn demux_bytes<N2: Location<'a>>(
        &self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let recv_ir = node_recv_direct(self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            node: other.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(recv_ir),
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: Location<'a>> Stream<'a, (u32, T), W, N> {
    pub fn demux_bincode<N2: Location<'a>>(self, other: &N2) -> Stream<'a, T, Async, N2>
    where
        N: HfSendOneToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        cluster_demux_bincode(&self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let recv_ir = node_recv_bincode::<_, T, _, _, _>(
            &self,
            other,
            N::gen_source_statement(other, &recv_port),
            false,
        );

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            node: other.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(recv_ir),
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }
}

impl<'a, W, N: Location<'a>> Stream<'a, (u32, Bytes), W, N> {
    pub fn demux_bytes_tagged<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<(u32, BytesMut), io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        node_send_direct(&self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let recv_ir = node_recv_direct(&self, other, N::gen_source_statement(other, &recv_port));

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            node: other.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(recv_ir),
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn demux_bytes_interleaved<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, Result<BytesMut, io::Error>, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        self.demux_bytes_tagged(other)
            .map(q!(|r| r.map(|(_, b)| b)))
    }
}

impl<'a, T: Serialize + DeserializeOwned, W, N: Location<'a>> Stream<'a, (u32, T), W, N> {
    pub fn demux_bincode_tagged<N2: Location<'a>>(
        self,
        other: &N2,
    ) -> Stream<'a, (u32, T), Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        let send_port = self.node.next_port();
        cluster_demux_bincode(&self, self.node.gen_sink_statement(&send_port));

        let recv_port = other.next_port();
        let recv_ir = node_recv_bincode::<_, T, _, _, _>(
            &self,
            other,
            N::gen_source_statement(other, &recv_port),
            true,
        );

        self.node.connect(other, &send_port, &recv_port);

        Stream {
            node: other.clone(),
            ir_leaves: self.ir_leaves,
            ir_node: RefCell::new(recv_ir),
            is_delta: self.is_delta,
            _phantom: PhantomData,
        }
    }

    pub fn demux_bincode_interleaved<N2: Location<'a>>(self, other: &N2) -> Stream<'a, T, Async, N2>
    where
        N: HfSendManyToMany<'a, N2>,
    {
        self.demux_bincode_tagged(other).map(q!(|(_, b)| b))
    }
}
