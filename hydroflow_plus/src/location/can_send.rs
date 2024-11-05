use stageleft::quote_type;

use super::{Cluster, ClusterId, ExternalProcess, Location, Process};

pub trait CanSend<'a, To: Location<'a>>: Location<'a> {
    type In<T>;
    type Out<T>;

    fn is_demux() -> bool;
    fn tagged_type() -> Option<syn::Type>;
}

impl<'a, P1, P2> CanSend<'a, Process<'a, P2>> for Process<'a, P1> {
    type In<T> = T;
    type Out<T> = T;

    fn is_demux() -> bool {
        false
    }

    fn tagged_type() -> Option<syn::Type> {
        None
    }
}

impl<'a, P1, C2> CanSend<'a, Cluster<'a, C2>> for Process<'a, P1> {
    type In<T> = (ClusterId<C2>, T);
    type Out<T> = T;

    fn is_demux() -> bool {
        true
    }

    fn tagged_type() -> Option<syn::Type> {
        None
    }
}

impl<'a, C1, P2> CanSend<'a, Process<'a, P2>> for Cluster<'a, C1> {
    type In<T> = T;
    type Out<T> = (ClusterId<C1>, T);

    fn is_demux() -> bool {
        false
    }

    fn tagged_type() -> Option<syn::Type> {
        Some(quote_type::<C1>())
    }
}

impl<'a, C1, C2> CanSend<'a, Cluster<'a, C2>> for Cluster<'a, C1> {
    type In<T> = (ClusterId<C2>, T);
    type Out<T> = (ClusterId<C1>, T);

    fn is_demux() -> bool {
        true
    }

    fn tagged_type() -> Option<syn::Type> {
        Some(quote_type::<C1>())
    }
}

impl<'a, P1, E2> CanSend<'a, ExternalProcess<'a, E2>> for Process<'a, P1> {
    type In<T> = T;
    type Out<T> = T;

    fn is_demux() -> bool {
        false
    }

    fn tagged_type() -> Option<syn::Type> {
        None
    }
}
