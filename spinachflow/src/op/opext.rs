use std::any::Any;
use std::net::SocketAddr;

use serde::ser::Serialize;
use tokio::net::tcp::OwnedWriteHalf;

use crate::comp::{DebugComp, NullComp, TcpComp, TcpServerComp};
use crate::func::unary::{Morphism, ClosureMorphism};
use crate::func::binary::BinaryMorphism;
use crate::lattice::{Convert, Debottom, LatticeRepr, Merge, Top};
use crate::lattice::map_union::{MapTag, MapUnionRepr};
use crate::lattice::pair::PairRepr;
use crate::tcp_server::TcpServer;
use crate::hide::{Hide, Delta};

use super::*;

impl<O> OpExt for O where O: Op {}

pub trait OpExt: Sized + Op {
    fn debug(self, tag: &'static str) -> DebugOp<Self>
    where
        <Self::LatRepr as LatticeRepr>::Repr: std::fmt::Debug,
    {
        DebugOp::new(self, tag)
    }

    fn debottom(self) -> DebottomOp<Self>
    where
        Self::LatRepr: Debottom,
    {
        DebottomOp::new(self)
    }

    fn morphism<F: Morphism<InLatRepr = Self::LatRepr>>(self, func: F) -> MorphismOp<Self, F> {
        MorphismOp::new(self, func)
    }

    fn morphism_closure<Out: LatticeRepr, F>(self, func: F) -> MorphismOp<Self, ClosureMorphism<Self::LatRepr, Out, F>>
    where
        F: Fn(Hide<Delta, Self::LatRepr>) -> Hide<Delta, Out>,
    {
        MorphismOp::new(self, ClosureMorphism::new(func))
    }

    fn topbox(self) -> TopOp<Self>
    where
        Self::LatRepr: Top,
    {
        TopOp::new(self)
    }

    fn lattice<Lr: LatticeRepr + Merge<Self::LatRepr>>(self, bottom: Lr::Repr) -> LatticeOp<Self, Lr>
    where
        Self::LatRepr: Convert<Lr>,
    {
        LatticeOp::new(self, bottom)
    }

    fn binary<F, O: OpValue>(self, op: O, func: F) -> BinaryOp<Self, O, F>
    where
        Self: OpValue,
        F: BinaryMorphism<InLatReprA = Self::LatRepr, InLatReprB = O::LatRepr>,
    {
        BinaryOp::new(self, op, func)
    }

    fn lattice_default<Lr: LatticeRepr + Merge<Self::LatRepr>>(self) -> LatticeOp<Self, Lr>
    where
        Self::LatRepr: Convert<Lr>,
        Lr::Repr: Default,
    {
        LatticeOp::new_default(self)
    }

    fn fixed_split<const N: usize>(self) -> [SplitOp<Self>; N] {
        fixed_split(self)
    }

    fn dyn_split(self) -> Splitter<Self>
    where
        Self: OpValue,
    {
        Splitter::new(self)
    }

    fn switch<Ra: LatticeRepr, Rb: LatticeRepr>(self) -> (SwitchOp<Self, Ra, Rb, switch::SwitchModeA>, SwitchOp<Self, Ra, Rb, switch::SwitchModeB>)
    where
        Self: Op<LatRepr = PairRepr<Ra, Rb>>,
    {
        SwitchOp::new(self)
    }

    fn comp_debug(self, tag: &'static str) -> DebugComp<Self>
    where
        Self: OpDelta,
        <Self::LatRepr as LatticeRepr>::Repr: std::fmt::Debug,
    {
        DebugComp::new(self, tag)
    }

    fn comp_null(self) -> NullComp<Self>
    where
        Self: OpDelta,
    {
        NullComp::new(self)
    }

    fn comp_tcp<Lr: Any + LatticeRepr>(self, tcp_write: OwnedWriteHalf) -> TcpComp<Self>
    where
        Self: OpDelta<LatRepr = Lr>,
        Lr::Repr: Serialize,
    {
        TcpComp::new(self, tcp_write)
    }

    fn comp_tcp_server<Lr: Any + LatticeRepr, Tag>(self, tcp_server: TcpServer) -> TcpServerComp<Self, Tag, Lr>
    where
        Tag: MapTag<SocketAddr, Lr::Repr>,
        MapUnionRepr<Tag, SocketAddr, Lr>: LatticeRepr,
        Self: OpDelta<LatRepr = MapUnionRepr<Tag, SocketAddr, Lr>>,
        <Self::LatRepr as LatticeRepr>::Repr: IntoIterator<Item = (SocketAddr, Lr::Repr)>,
        Lr::Repr: Serialize,
    {
        TcpServerComp::new(self, tcp_server)
    }
}
