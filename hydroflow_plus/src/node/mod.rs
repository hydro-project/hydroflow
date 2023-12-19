use std::cell::RefCell;
use std::io;
use std::marker::PhantomData;

use hydroflow::bytes::BytesMut;
use hydroflow::futures::stream::Stream;
use proc_macro2::Span;
use stageleft::Quoted;
use syn::parse_quote;

use crate::builder::Builders;
use crate::{HfBuilder, HfCycle, HfStream};

mod graphs;
pub use graphs::*;

pub mod network;
pub use network::*;

pub trait HfDeploy<'a> {
    type Node: HfNode<'a, Meta = Self::Meta>;
    type Cluster: HfNode<'a, Meta = Self::Meta> + HfCluster<'a>;
    type Meta;
    type RuntimeID;
}

pub trait HfNetworkedDeploy<'a>:
    HfDeploy<'a, Node = Self::NetworkedNode, Cluster = Self::NetworkedCluster>
{
    type NetworkedNode: HfNode<'a, Port = Self::NodePort>
        + HfSendOneToOne<'a, Self::NetworkedNode>
        + HfSendOneToMany<'a, Self::NetworkedCluster>;
    type NetworkedCluster: HfNode<'a, Port = Self::ClusterPort>
        + HfSendManyToOne<'a, Self::NetworkedNode>
        + HfSendManyToMany<'a, Self::NetworkedCluster>
        + HfCluster<'a>;
    type NodePort;
    type ClusterPort;
}

impl<
        'a,
        T: HfDeploy<'a, Node = N, Cluster = C>,
        N: HfNode<'a> + HfSendOneToOne<'a, N> + HfSendOneToMany<'a, C>,
        C: HfNode<'a> + HfSendManyToOne<'a, N> + HfSendManyToMany<'a, C> + HfCluster<'a>,
    > HfNetworkedDeploy<'a> for T
{
    type NetworkedNode = N;
    type NetworkedCluster = C;
    type NodePort = N::Port;
    type ClusterPort = C::Port;
}

pub trait HfNodeBuilder<'a, D: HfDeploy<'a> + ?Sized> {
    fn build(&self, id: usize, builder: &'a HfBuilder<'a, D>) -> D::Node;
}

pub trait HfClusterBuilder<'a, D: HfDeploy<'a> + ?Sized> {
    fn build(&self, id: usize, builder: &'a HfBuilder<'a, D>) -> D::Cluster;
}

pub trait HfNode<'a>: Clone {
    type Port;
    type Meta;

    fn id(&self) -> usize;
    fn graph_builder(&self) -> (&'a RefCell<usize>, &'a Builders);
    fn next_port(&self) -> Self::Port;

    fn build(&mut self, meta: &Option<Self::Meta>);

    fn source_stream<T, E: Stream<Item = T> + Unpin>(
        &self,
        e: impl Quoted<'a, E>,
    ) -> HfStream<'a, T, Self> {
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

        HfStream {
            ident,
            node: self.clone(),
            next_id: next_id_cell,
            builders,
            _phantom: PhantomData,
        }
    }

    fn source_external(&self) -> (Self::Port, HfStream<'a, Result<BytesMut, io::Error>, Self>)
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
            HfStream {
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
    ) -> HfStream<'a, T, Self> {
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

        HfStream {
            ident,
            node: self.clone(),
            next_id: next_id_cell,
            builders,
            _phantom: PhantomData,
        }
    }

    fn cycle<T>(&self) -> (HfCycle<'a, T, Self>, HfStream<'a, T, Self>) {
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
            HfStream {
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
