use super::Location;

pub trait HfSend<'a, O: Location<'a>, V>: Location<'a> {
    type In<T>;
    type Out<T>;

    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;

    fn is_demux() -> bool;
    fn is_tagged() -> bool;
}

pub trait HfSendOneToOne<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<'a, O: Location<'a>, H: HfSendOneToOne<'a, O>> HfSend<'a, O, ()> for H {
    type In<T> = T;
    type Out<T> = T;

    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port) {
        H::connect(self, other, source_port, recipient_port);
    }

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr {
        H::gen_sink_statement(self, port)
    }

    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr {
        H::gen_source_statement(other, port)
    }

    fn is_demux() -> bool {
        false
    }

    fn is_tagged() -> bool {
        false
    }
}

pub trait HfSendManyToOne<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<'a, O: Location<'a>, H: HfSendManyToOne<'a, O>> HfSend<'a, O, ((), ())> for H {
    type In<T> = T;
    type Out<T> = (u32, T);

    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port) {
        H::connect(self, other, source_port, recipient_port);
    }

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr {
        H::gen_sink_statement(self, port)
    }

    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr {
        H::gen_source_statement(other, port)
    }

    fn is_demux() -> bool {
        false
    }

    fn is_tagged() -> bool {
        true
    }
}

pub trait HfSendOneToMany<'a, O: Location<'a>, Cid>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<'a, O: Location<'a>, Cid, H: HfSendOneToMany<'a, O, Cid>> HfSend<'a, O, ((), (), Cid)> for H {
    type In<T> = (Cid, T);
    type Out<T> = T;

    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port) {
        H::connect(self, other, source_port, recipient_port);
    }

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr {
        H::gen_sink_statement(self, port)
    }

    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr {
        H::gen_source_statement(other, port)
    }

    fn is_demux() -> bool {
        true
    }

    fn is_tagged() -> bool {
        false
    }
}

pub trait HfSendManyToMany<'a, O: Location<'a>, Cid>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<'a, O: Location<'a>, Cid, H: HfSendManyToMany<'a, O, Cid>> HfSend<'a, O, ((), (), Cid, Cid)> for H {
    type In<T> = (Cid, T);
    type Out<T> = (Cid, T);

    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port) {
        H::connect(self, other, source_port, recipient_port);
    }

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr {
        H::gen_sink_statement(self, port)
    }

    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr {
        H::gen_source_statement(other, port)
    }

    fn is_demux() -> bool {
        true
    }

    fn is_tagged() -> bool {
        true
    }
}
