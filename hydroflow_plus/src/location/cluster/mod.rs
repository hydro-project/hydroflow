use std::marker::PhantomData;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use stageleft::runtime_support::FreeVariableWithContext;
use stageleft::{quote_type, QuotedWithContext};

use super::{Location, LocationId};
use crate::builder::FlowState;
use crate::staging_util::{get_this_crate, Invariant};

pub mod cluster_id;
pub use cluster_id::ClusterId;

pub struct Cluster<'a, C> {
    pub(crate) id: usize,
    pub(crate) flow_state: FlowState,
    pub(crate) _phantom: Invariant<'a, C>,
}

impl<'a, C> Cluster<'a, C> {
    pub fn members(&self) -> ClusterIds<'a, C> {
        ClusterIds {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<C> Clone for Cluster<'_, C> {
    fn clone(&self) -> Self {
        Cluster {
            id: self.id,
            flow_state: self.flow_state.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, C> Location<'a> for Cluster<'a, C> {
    type Root = Cluster<'a, C>;

    fn root(&self) -> Self::Root {
        self.clone()
    }

    fn id(&self) -> LocationId {
        LocationId::Cluster(self.id)
    }

    fn flow_state(&self) -> &FlowState {
        &self.flow_state
    }

    fn is_top_level() -> bool {
        true
    }
}

pub struct ClusterIds<'a, C> {
    pub(crate) id: usize,
    _phantom: Invariant<'a, C>,
}

impl<C> Clone for ClusterIds<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for ClusterIds<'_, C> {}

impl<'a, C: 'a, Ctx> FreeVariableWithContext<Ctx> for ClusterIds<'a, C> {
    type O = &'a Vec<ClusterId<C>>;

    fn to_tokens(self, _ctx: &Ctx) -> (Option<TokenStream>, Option<TokenStream>)
    where
        Self: Sized,
    {
        let ident = syn::Ident::new(
            &format!("__hydroflow_plus_cluster_ids_{}", self.id),
            Span::call_site(),
        );
        let root = get_this_crate();
        let c_type = quote_type::<C>();
        (
            None,
            Some(
                quote! { unsafe { ::std::mem::transmute::<_, &::std::vec::Vec<#root::ClusterId<#c_type>>>(#ident) } },
            ),
        )
    }
}

impl<'a, C, Ctx> QuotedWithContext<'a, &'a Vec<ClusterId<C>>, Ctx> for ClusterIds<'a, C> {}

/// A free variable representing the cluster's own ID. When spliced in
/// a quoted snippet that will run on a cluster, this turns into a [`ClusterId`].
pub static CLUSTER_SELF_ID: ClusterSelfId = ClusterSelfId { _private: () };

#[derive(Clone, Copy)]
pub struct ClusterSelfId {
    pub(crate) _private: (),
}

impl<'a, C> FreeVariableWithContext<Cluster<'a, C>> for ClusterSelfId {
    type O = ClusterId<C>;

    fn to_tokens(self, ctx: &Cluster<'a, C>) -> (Option<TokenStream>, Option<TokenStream>)
    where
        Self: Sized,
    {
        let ident = syn::Ident::new(
            &format!("__hydroflow_plus_cluster_self_id_{}", ctx.id),
            Span::call_site(),
        );
        let root = get_this_crate();
        let c_type: syn::Type = quote_type::<C>();
        (
            None,
            Some(quote! { #root::ClusterId::<#c_type>::from_raw(#ident) }),
        )
    }
}

impl<'a, C> QuotedWithContext<'a, ClusterId<C>, Cluster<'a, C>> for ClusterSelfId {}
