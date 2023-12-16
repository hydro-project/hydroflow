use std::io;
use std::marker::PhantomData;

use hydroflow::bytes::BytesMut;
use hydroflow::futures::stream::Stream;
use hydroflow_lang::parse::Pipeline;
use proc_macro2::Span;
use stageleft::Quoted;
use syn::parse_quote;

use crate::{HfBuilder, HfCycle, HfStream};

mod graphs;
pub use graphs::*;

pub trait HfDeploy<'a> {
    type Node: HfNode<'a, Self>;
}

pub trait HfNetworkedDeploy<'a>: HfDeploy<'a, Node = Self::NetworkedNode> {
    type NetworkedNode: HfNode<'a, Self, Port = Self::Port>
        + HfSendTo<'a, Self, Self::NetworkedNode>;
    type Port;
}

impl<'a, T: HfDeploy<'a, Node = N>, N: HfSendTo<'a, T, N>> HfNetworkedDeploy<'a> for T {
    type NetworkedNode = N;
    type Port = N::Port;
}

pub trait HfNodeBuilder<'a, D: HfDeploy<'a> + ?Sized> {
    fn build(&mut self, id: usize, builder: &'a HfBuilder<'a, D>) -> D::Node;
}

pub trait HfSendTo<'a, D: HfDeploy<'a> + ?Sized, O: HfNode<'a, D>>: HfNode<'a, D> {
    fn send_to(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);
}

pub trait HfNode<'a, D: HfDeploy<'a> + ?Sized>: Clone {
    type Port;

    fn id(&self) -> usize;
    fn builder(&self) -> &'a HfBuilder<'a, D>;
    fn next_port(&self) -> Self::Port;
    fn gen_source_statement(&self, port: &Self::Port) -> Pipeline;
    fn gen_sink_statement(&self, port: &Self::Port) -> Pipeline;

    fn source_stream<T, E: Stream<Item = T> + Unpin>(
        &self,
        e: impl Quoted<'a, E>,
    ) -> HfStream<'a, T, D, D::Node>
    where
        D: HfDeploy<'a, Node = Self>,
    {
        let graph = self.builder();

        let next_id = {
            let mut next_id = graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let e = e.splice();

        graph
            .builders
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
            graph,
            _phantom: PhantomData,
        }
    }

    fn source_external(
        &self,
    ) -> (
        Self::Port,
        HfStream<'a, Result<BytesMut, io::Error>, D, Self>,
    )
    where
        D: HfDeploy<'a, Node = Self>,
    {
        let graph = self.builder();

        let next_id = {
            let mut next_id = graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let port = self.next_port();
        let source_pipeline = self.gen_source_statement(&port);

        graph
            .builders
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
                graph,
                _phantom: PhantomData,
            },
        )
    }

    fn source_iter<T, E: IntoIterator<Item = T>>(
        &self,
        e: impl Quoted<'a, E>,
    ) -> HfStream<'a, T, D, D::Node>
    where
        D: HfDeploy<'a, Node = Self>,
    {
        let graph = self.builder();

        let next_id = {
            let mut next_id = graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());
        let e = e.splice();

        graph
            .builders
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
            graph,
            _phantom: PhantomData,
        }
    }

    #[allow(clippy::type_complexity)]
    fn cycle<T>(&self) -> (HfCycle<'a, T, D, D::Node>, HfStream<'a, T, D, D::Node>)
    where
        D: HfDeploy<'a, Node = Self>,
    {
        let graph = self.builder();

        let next_id = {
            let mut next_id = graph.next_id.borrow_mut();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let ident = syn::Ident::new(&format!("stream_{}", next_id), Span::call_site());

        graph
            .builders
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
                graph,
                _phantom: PhantomData,
            },
            HfStream {
                ident,
                node: self.clone(),
                graph,
                _phantom: PhantomData,
            },
        )
    }
}
