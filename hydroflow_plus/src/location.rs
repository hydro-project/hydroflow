use std::marker::PhantomData;

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

pub struct Cluster<C> {
    pub(crate) id: usize,
    pub(crate) _phantom: PhantomData<C>,
}

impl<C> Clone for Cluster<C> {
    fn clone(&self) -> Self {
        Cluster {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<C> Location for Cluster<C> {
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

impl<P1, C2> CanSend<Cluster<C2>> for Process<P1> {
    type In<T> = (u32, T);
    type Out<T> = T;

    fn is_demux() -> bool {
        true
    }

    fn is_tagged() -> bool {
        false
    }
}

impl<C1, P2> CanSend<Process<P2>> for Cluster<C1> {
    type In<T> = T;
    type Out<T> = (u32, T);

    fn is_demux() -> bool {
        false
    }

    fn is_tagged() -> bool {
        true
    }
}

impl<C1, C2> CanSend<Cluster<C2>> for Cluster<C1> {
    type In<T> = (u32, T);
    type Out<T> = (u32, T);

    fn is_demux() -> bool {
        true
    }

    fn is_tagged() -> bool {
        true
    }
}
