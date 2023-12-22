use std::cell::RefCell;
use std::io;
use std::marker::PhantomData;

use hydroflow::bytes::BytesMut;
use hydroflow::futures::stream::Stream as FuturesStream;
use proc_macro2::Span;
use stageleft::Quoted;
use syn::parse_quote;

use crate::builder::Builders;
use crate::stream::{Async, Windowed};
use crate::{GraphBuilder, HfCycle, Stream};

mod graphs;
pub use graphs::*;

pub mod network;
pub use network::*;

pub trait LocalDeploy<'a> {
    type Node: HfNode<'a, Meta = Self::Meta>;
    type Cluster: HfNode<'a, Meta = Self::Meta> + HfCluster<'a>;
    type Meta: Default;
    type RuntimeID;
}

pub trait Deploy<'a> {
    type Node: HfNode<'a, Meta = Self::Meta, Port = Self::NodePort>
        + HfSendOneToOne<'a, Self::Node>
        + HfSendOneToMany<'a, Self::Cluster>;
    type Cluster: HfNode<'a, Meta = Self::Meta, Port = Self::ClusterPort>
        + HfSendManyToOne<'a, Self::Node>
        + HfSendManyToMany<'a, Self::Cluster>
        + HfCluster<'a>;
    type NodePort;
    type ClusterPort;
    type Meta: Default;
    type RuntimeID;
}

impl<
        'a,
        T: Deploy<'a, Node = N, Cluster = C, Meta = M, RuntimeID = R>,
        N: HfNode<'a, Meta = M> + HfSendOneToOne<'a, N> + HfSendOneToMany<'a, C>,
        C: HfNode<'a, Meta = M> + HfSendManyToOne<'a, N> + HfSendManyToMany<'a, C> + HfCluster<'a>,
        M: Default,
        R,
    > LocalDeploy<'a> for T
{
    type Node = N;
    type Cluster = C;
    type Meta = M;
    type RuntimeID = R;
}

pub trait NodeBuilder<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(&self, id: usize, builder: &'a GraphBuilder<'a, D>, meta: &mut D::Meta) -> D::Node;
}

pub trait ClusterBuilder<'a, D: LocalDeploy<'a> + ?Sized> {
    fn build(&self, id: usize, builder: &'a GraphBuilder<'a, D>, meta: &mut D::Meta) -> D::Cluster;
}

pub trait HfNode<'a>: Clone {
    type Port;
    type Meta;

    fn id(&self) -> usize;
    fn graph_builder(&self) -> (&'a RefCell<usize>, &'a Builders);
    fn next_port(&self) -> Self::Port;

    fn update_meta(&mut self, meta: &Self::Meta);

    fn source_stream<T, E: FuturesStream<Item = T> + Unpin>(
        &self,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Async, Self> {
        let (next_id_cell, builders) = self.graph_builder();

        let next_id = {
            let mut next_id = next_id_cell.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let e = e.splice();

        builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = source_stream(#e) -> tee();
            });

        Stream {
            ident,
            node: self.clone(),
            next_id: next_id_cell,
            builders,
            _phantom: PhantomData,
        }
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
        let (next_id_cell, builders) = self.graph_builder();

        let next_id = {
            let mut next_id = next_id_cell.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let port = self.next_port();
        let source_pipeline = Self::gen_source_statement(self, &port);

        builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = #source_pipeline -> tee();
            });

        (
            port,
            Stream {
                ident,
                node: self.clone(),
                next_id: next_id_cell,
                builders,
                _phantom: PhantomData,
            },
        )
    }

    fn source_iter<T, E: IntoIterator<Item = T>>(
        &self,
        e: impl Quoted<'a, E>,
    ) -> Stream<'a, T, Windowed, Self> {
        let (next_id_cell, builders) = self.graph_builder();

        let next_id = {
            let mut next_id = next_id_cell.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let e = e.splice();

        builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = source_iter(#e) -> tee();
            });

        Stream {
            ident,
            node: self.clone(),
            next_id: next_id_cell,
            builders,
            _phantom: PhantomData,
        }
    }

    fn cycle<T, W>(&self) -> (HfCycle<'a, T, W, Self>, Stream<'a, T, W, Self>) {
        let (next_id_cell, builders) = self.graph_builder();

        let next_id = {
            let mut next_id = next_id_cell.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        builders
            .borrow_mut()
            .as_mut()
            .unwrap()
            .entry(self.id())
            .or_default()
            .add_statement(parse_quote! {
                #ident = tee();
            });

        (
            HfCycle {
                ident: ident.clone(),
                node: self.clone(),
                builders,
                _phantom: PhantomData,
            },
            Stream {
                ident,
                node: self.clone(),
                next_id: next_id_cell,
                builders,
                _phantom: PhantomData,
            },
        )
    }
}

pub trait HfCluster<'a> {
    fn ids<'b>(&'b self) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a;
}
