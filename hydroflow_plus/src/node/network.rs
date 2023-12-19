use hydroflow_lang::parse::Pipeline;

use super::HfNode;

pub trait HfSendOneToOne<'a, O: HfNode<'a>>: HfNode<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> Pipeline;
    fn gen_source_statement(other: &O, port: &O::Port) -> Pipeline;
}

pub trait HfSendManyToOne<'a, O: HfNode<'a>>: HfNode<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> Pipeline;
    fn gen_source_statement(other: &O, port: &O::Port) -> Pipeline;
}

pub trait HfSendOneToMany<'a, O: HfNode<'a>>: HfNode<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> Pipeline;
    fn gen_source_statement(other: &O, port: &O::Port) -> Pipeline;
}

pub trait HfSendManyToMany<'a, O: HfNode<'a>>: HfNode<'a> {
    fn connect(&self, other: &O, source_port: &Self::Port, recipient_port: &O::Port);

    fn gen_sink_statement(&self, port: &Self::Port) -> Pipeline;
    fn gen_source_statement(other: &O, port: &O::Port) -> Pipeline;
}
