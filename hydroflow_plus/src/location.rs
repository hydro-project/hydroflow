use std::marker::PhantomData;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use stageleft::runtime_support::FreeVariable;
use stageleft::Quoted;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LocationId {
    Process(usize),
    Cluster(usize),
}

pub trait Location {
    fn id(&self) -> LocationId;
}

pub struct Process<P> {
    pub(crate) id: usize,
    pub(crate) _phantom: PhantomData<P>,
}

impl<P> Clone for Process<P> {
    fn clone(&self) -> Self {
        Process {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<P> Location for Process<P> {
    fn id(&self) -> LocationId {
        LocationId::Process(self.id)
    }
}

pub struct Cluster<'a, C> {
    pub(crate) id: usize,
    pub(crate) _phantom: PhantomData<&'a mut &'a C>,
}

#[derive(Copy, Clone)]
struct ClusterIds<'a> {
    id: usize,
    _phantom: PhantomData<&'a mut &'a Vec<u32>>,
}

impl<'a> FreeVariable<&'a Vec<u32>> for ClusterIds<'a> {
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>)
    where
        Self: Sized,
    {
        let ident = syn::Ident::new(
            &format!("__hydroflow_plus_cluster_ids_{}", self.id),
            Span::call_site(),
        );
        (None, Some(quote! { #ident }))
    }
}

impl<'a> Quoted<'a, &'a Vec<u32>> for ClusterIds<'a> {}

#[derive(Copy, Clone)]
struct ClusterSelfId<'a> {
    id: usize,
    _phantom: PhantomData<&'a mut &'a u32>,
}

impl<'a> FreeVariable<u32> for ClusterSelfId<'a> {
    fn to_tokens(self) -> (Option<TokenStream>, Option<TokenStream>)
    where
        Self: Sized,
    {
        let ident = syn::Ident::new(
            &format!("__hydroflow_plus_cluster_self_id_{}", self.id),
            Span::call_site(),
        );
        (None, Some(quote! { #ident }))
    }
}

impl<'a> Quoted<'a, u32> for ClusterSelfId<'a> {}

impl<'a, C> Cluster<'a, C> {
    pub fn ids(&self) -> impl Quoted<'a, &'a Vec<u32>> + Copy + 'a {
        ClusterIds {
            id: self.id,
            _phantom: PhantomData,
        }
    }

    pub fn self_id(&self) -> impl Quoted<'a, u32> + Copy + 'a {
        ClusterSelfId {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<'a, C> Clone for Cluster<'a, C> {
    fn clone(&self) -> Self {
        Cluster {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<'a, C> Location for Cluster<'a, C> {
    fn id(&self) -> LocationId {
        LocationId::Cluster(self.id)
    }
}

pub trait CanSend<To: Location>: Location {
    type In<T>;
    type Out<T>;

    fn is_demux() -> bool;
    fn is_tagged() -> bool;
}

impl<P1, P2> CanSend<Process<P2>> for Process<P1> {
    type In<T> = T;
    type Out<T> = T;

    fn is_demux() -> bool {
        false
    }

    fn is_tagged() -> bool {
        false
    }
}

impl<'a, P1, C2> CanSend<Cluster<'a, C2>> for Process<P1> {
    type In<T> = (u32, T);
    type Out<T> = T;

    fn is_demux() -> bool {
        true
    }

    fn is_tagged() -> bool {
        false
    }
}

impl<'a, C1, P2> CanSend<Process<P2>> for Cluster<'a, C1> {
    type In<T> = T;
    type Out<T> = (u32, T);

    fn is_demux() -> bool {
        false
    }

    fn is_tagged() -> bool {
        true
    }
}

impl<'a, C1, C2> CanSend<Cluster<'a, C2>> for Cluster<'a, C1> {
    type In<T> = (u32, T);
    type Out<T> = (u32, T);

    fn is_demux() -> bool {
        true
    }

    fn is_tagged() -> bool {
        true
    }
}
