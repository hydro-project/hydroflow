use super::Location;

pub trait HfSendOneToOne<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

pub trait HfSendManyToOne<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

pub trait HfSendOneToMany<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}

pub trait HfSendManyToMany<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> syn::Expr;
    fn gen_source_statement(other: &O, port: &O::Port) -> syn::Expr;
}
