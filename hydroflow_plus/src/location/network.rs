use super::Location;

pub trait HfSendToOne<'a, O: Location<'a>, V>: Location<'a> {
    type Out<T>;

    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;

    fn is_tagged() -> bool;
}

pub trait HfSendOneToOne<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<'a, O: Location<'a>, H: HfSendOneToOne<'a, O>> HfSendToOne<'a, O, ()> for H {
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

    fn is_tagged() -> bool {
        false
    }
}

pub trait HfSendManyToOne<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<'a, O: Location<'a>, H: HfSendManyToOne<'a, O>> HfSendToOne<'a, O, u32> for H {
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

    fn is_tagged() -> bool {
        true
    }
}

pub trait HfSendToMany<'a, O: Location<'a>, Cid, V>: Location<'a> {
    type Out<T>;

    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;

    fn is_tagged() -> bool;
}

pub trait HfSendOneToMany<'a, O: Location<'a>, Cid>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<'a, O: Location<'a>, Cid, H: HfSendOneToMany<'a, O, Cid>> HfSendToMany<'a, O, Cid, ()> for H {
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

    fn is_tagged() -> bool {
        false
    }
}

pub trait HfSendManyToMany<'a, O: Location<'a>, Cid>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

impl<'a, O: Location<'a>, Cid, H: HfSendManyToMany<'a, O, Cid>> HfSendToMany<'a, O, Cid, u32>
    for H
{
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

    fn is_tagged() -> bool {
        true
    }
}
