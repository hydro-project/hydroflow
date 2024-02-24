use proc_macro2::TokenStream;

use super::Location;

pub trait HfSendOneToOne<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> TokenStream;
    fn gen_source_statement(other: &O, port: &O::Port) -> TokenStream;
}

pub trait HfSendManyToOne<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> TokenStream;
    fn gen_source_statement(other: &O, port: &O::Port) -> TokenStream;
}

pub trait HfSendOneToMany<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> TokenStream;
    fn gen_source_statement(other: &O, port: &O::Port) -> TokenStream;
}

pub trait HfSendManyToMany<'a, O: Location<'a>>: Location<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> TokenStream;
    fn gen_source_statement(other: &O, port: &O::Port) -> TokenStream;
}
