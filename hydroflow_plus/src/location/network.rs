use super::Location;

pub trait HfSend<O: Location, V>: Location {
    type In<T>;
    type Out<T>;

    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;

    fn is_demux() -> bool;
    fn is_tagged() -> bool;
}

pub trait HfSendOneToOne<O: Location>: Location {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<O: Location, H: HfSendOneToOne<O>> HfSend<O, ()> for H {
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

pub trait HfSendManyToOne<O: Location, Tag>: Location {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<O: Location, Tag, H: HfSendManyToOne<O, Tag>> HfSend<O, ((), Tag)> for H {
    type In<T> = T;
    type Out<T> = (Tag, T);

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

pub trait HfSendOneToMany<O: Location, Cid>: Location {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<O: Location, Cid, H: HfSendOneToMany<O, Cid>> HfSend<O, ((), (), Cid)> for H {
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

pub trait HfSendManyToMany<O: Location, Cid>: Location {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<O: Location, Cid, H: HfSendManyToMany<O, Cid>> HfSend<O, ((), (), Cid, Cid)> for H {
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
