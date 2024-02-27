use std::cell::RefCell;
use std::io;
use std::marker::PhantomData;
use std::time::Duration;

use hydroflow::bytes::BytesMut;
use hydroflow::futures::stream::Stream as FuturesStream;
use proc_macro2::Span;
use stageleft::{q, Quoted};

use crate::ir::{HfPlusLeaf, HfPlusNode, HfPlusSource};
use crate::stream::{Async, Windowed};
use crate::{FlowBuilder, HfCycle, Stream};

pub mod graphs;
pub use graphs::*;

pub mod network;
pub use network::*;

pub trait LocalDeploy<'a> {
    type Process: Location<'a, Meta = Self::Meta>;
    type Cluster: Location<'a, Meta = Self::Meta> + Cluster<'a>;
    type Meta: Default;
    type RuntimeID;
}

pub trait Deploy<'a> {
    type Process: Location<'a, Meta = Self::Meta, Port = Self::ProcessPort>
        + HfSendOneToOne<'a, Self::Process>
        + HfSendOneToMany<'a, Self::Cluster>;
    type Cluster: Location<'a, Meta = Self::Meta, Port = Self::ClusterPort>
        + HfSendManyToOne<'a, Self::Process>
        + HfSendManyToMany<'a, Self::Cluster>
        + Cluster<'a>;
    type ProcessPort;
    type ClusterPort;
    type Meta: Default;
    type RuntimeID;
}

impl<
        'a,
        T: Deploy<'a, Process = N, Cluster = C, Meta = M, RuntimeID = R>,
        N: Location<'a, Meta = M> + HfSendOneToOne<'a, N> + HfSendOneToMany<'a, C>,
        C: Location<'a, Meta = M> + HfSendManyToOne<'a, N> + HfSendManyToMany<'a, C> + Cluster<'a>,
        M: Default,
        R,
    > LocalDeploy<'a> for T
{
    type Process = N;
    type Cluster = C;
    type Meta = M;
    type RuntimeID = R;
}

pub trait ProcessSpec<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(&self, id: usize, builder: &'a FlowBuilder<'a, D>, meta: &mut D::Meta) -> D::Process;
}

pub trait ClusterSpec<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(&self, id: usize, builder: &'a FlowBuilder<'a, D>, meta: &mut D::Meta) -> D::Cluster;
}

pub trait Location<'a>: Clone {
    type Port;
    type Meta;

    fn id(&self) -> usize;
    fn flow_builder(&self) -> (&'a RefCell<usize>, &'a RefCell<Vec<HfPlusLeaf>>);
    fn next_port(&self) -> Self::Port;

    fn update_meta(&mut self, meta: &Self::Meta);

    fn spin(&self) -> Stream<'a, (), Async, Self> {
        let (_, ir_leaves) = self.flow_builder();

        Stream::new(
            self.clone(),
            ir_leaves,
            HfPlusNode::Source {
                source: HfPlusSource::Spin(),
                produces_delta: false,
                location_id: self.id(),
            },
        )
    }

    fn spin_batch(
        &self,
        batch_size: impl Quoted<'a, usize> + Copy + 'a,
    ) -> Stream<'a, (), Windowed, Self> {
        self.spin()
            .flat_map(q!(move |_| 0..batch_size))
            .map(q!(|_| ()))
            .tick_batch()
    }

    fn source_stream<T, E: FuturesStream<Item = T> + Unpin>(
        &self,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Async, Self> {
        let (_, ir_leaves) = self.flow_builder();

        let e = e.splice();

        Stream::new(
            self.clone(),
            ir_leaves,
            HfPlusNode::Source {
                source: HfPlusSource::Stream(e.into()),
                location_id: self.id(),
                produces_delta: false,
            },
        )
    }

    fn source_external(
        &self,
    ) -> (
        Self::Port,
        Stream<'a, Result<BytesMut, io::Error>, Async, Self>,
    )
    where
        Self: HfSendOneToOne<'a, Self>,
    {
        let (_, ir_leaves) = self.flow_builder();

        let port = self.next_port();
        let source_pipeline = Self::gen_source_statement(self, &port);

        (
            port,
            Stream::new(
                self.clone(),
                ir_leaves,
                HfPlusNode::Source {
                    source: HfPlusSource::Stream(source_pipeline.into()),
                    location_id: self.id(),
                    produces_delta: false,
                },
            ),
        )
    }

    fn many_source_external<S: Location<'a>>(
        &self,
    ) -> (
        Self::Port,
        Stream<'a, Result<BytesMut, io::Error>, Async, Self>,
    )
    where
        S: HfSendOneToMany<'a, Self>,
    {
        let (_, ir_leaves) = self.flow_builder();

        let port = self.next_port();
        let source_pipeline = S::gen_source_statement(self, &port);

        (
            port,
            Stream::new(
                self.clone(),
                ir_leaves,
                HfPlusNode::Source {
                    source: HfPlusSource::Stream(source_pipeline.into()),
                    location_id: self.id(),
                    produces_delta: false,
                },
            ),
        )
    }

    fn source_iter<T, E: IntoIterator<Item = T>>(
        &self,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Windowed, Self> {
        let (_, ir_leaves) = self.flow_builder();

        let e = e.splice();

        Stream::new(
            self.clone(),
            ir_leaves,
            HfPlusNode::Source {
                source: HfPlusSource::Iter(e.into()),
                location_id: self.id(),
                produces_delta: false,
            },
        )
    }

    fn source_interval(
        &self,
        interval: impl Quoted<'a, Duration> + Copy + 'a,
    ) -> Stream<'a, hydroflow::tokio::time::Instant, Async, Self> {
        let (_, ir_leaves) = self.flow_builder();

        let interval = interval.splice();

        Stream::new(
            self.clone(),
            ir_leaves,
            HfPlusNode::Source {
                source: HfPlusSource::Interval(interval.into()),
                location_id: self.id(),
                produces_delta: false,
            },
        )
    }

    fn cycle<T, W>(&self) -> (HfCycle<'a, T, W, Self>, Stream<'a, T, W, Self>) {
        let (next_id_cell, ir_leaves) = self.flow_builder();

        let next_id = {
            let mut next_id = next_id_cell.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("cycle_{}", next_id), Span::call_site());

        (
            HfCycle {
                ident: ident.clone(),
                node: self.clone(),
                ir_leaves,
                _phantom: PhantomData,
            },
            Stream::new(
                self.clone(),
                ir_leaves,
                HfPlusNode::CycleSource {
                    ident,
                    location_id: self.id(),
                },
            ),
        )
    }
}

pub trait Cluster<'a>: Location<'a> {
    fn ids<'b>(&'b self) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a;
}
