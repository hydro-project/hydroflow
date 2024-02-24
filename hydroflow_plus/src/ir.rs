use std::{cell::RefCell, collections::BTreeMap, ops::Deref, rc::Rc};

use hydroflow_lang::{graph::FlatGraphBuilder, parse::Pipeline};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse_quote;

pub struct DebugExpr(pub syn::Expr);

impl From<syn::Expr> for DebugExpr {
    fn from(expr: syn::Expr) -> DebugExpr {
        DebugExpr(expr)
    }
}

impl Deref for DebugExpr {
    type Target = syn::Expr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToTokens for DebugExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl std::fmt::Debug for DebugExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_token_stream().to_string())
    }
}

#[derive(Debug)]
pub enum HfPlusSource {
    Stream(DebugExpr),
    Iter(DebugExpr)
}

#[derive(Debug)]
pub enum HfPlusNode {
    Todo,
    Source {
        source: HfPlusSource,
        location_id: usize
    },
    Tee {
        inner: Rc<RefCell<HfPlusNode>>
    },
    CrossProduct(Box<HfPlusNode>, Box<HfPlusNode>),
    // TODO(shadaj): separate types for operators that don't return anything?
    ForEach {
        f: DebugExpr,
        input: Box<HfPlusNode>,
    },
    DestSink {
        sink: DebugExpr,
        input: Box<HfPlusNode>,
    },
    PipelineOp {
        pipeline: Pipeline,
        input: Box<HfPlusNode>
    },
}

impl HfPlusNode {
    pub fn emit(&self, graph_builders: &mut BTreeMap<usize, FlatGraphBuilder>, next_stmt_id: &mut usize) -> Option<(syn::Ident, usize)> {
        match self {
            HfPlusNode::Todo => {
                todo!()
            }
            HfPlusNode::Source { source, location_id }=> {
                let source_id = *next_stmt_id;
                *next_stmt_id += 1;

                let source_ident = syn::Ident::new(&format!("stream_{}", source_id), Span::call_site());

                let source_stmt = match source {
                    HfPlusSource::Stream(expr) => {
                        parse_quote! {
                            #source_ident = source_stream(#expr);
                        }
                    }
                    HfPlusSource::Iter(expr) => {
                        parse_quote! {
                            #source_ident = source_iter(#expr);
                        }
                    }
                };

                graph_builders.entry(*location_id).or_default()
                    .add_statement(source_stmt);

                Some((source_ident, *location_id))
            }

            HfPlusNode::Tee { inner: _inner } => {
                todo!()
            }

            HfPlusNode::CrossProduct(left, right) => {
                let (left_ident, left_location_id) = left.emit(graph_builders, next_stmt_id).unwrap();
                let (right_ident, right_location_id) = right.emit(graph_builders, next_stmt_id).unwrap();

                assert_eq!(left_location_id, right_location_id, "cross product inputs must be in the same location");

                let cross_product_id = *next_stmt_id;
                *next_stmt_id += 1;

                let cross_product_ident = syn::Ident::new(&format!("stream_{}", cross_product_id), Span::call_site());

                let builder = graph_builders.entry(left_location_id).or_default();
                builder.add_statement(parse_quote! {
                    #cross_product_ident = cross_join();
                });

                builder.add_statement(parse_quote! {
                    #left_ident -> [0]#cross_product_ident;
                });

                builder.add_statement(parse_quote! {
                    #right_ident -> [1]#cross_product_ident;
                });

                Some((cross_product_ident, left_location_id))
            }

            HfPlusNode::ForEach { f, input } => {
                let (input_ident, input_location_id) = input.emit(graph_builders, next_stmt_id).unwrap();

                graph_builders.entry(input_location_id).or_default()
                    .add_statement(parse_quote! {
                        #input_ident -> for_each(#f);
                    });

                None
            }
            HfPlusNode::DestSink { sink, input } => {
                let (input_ident, input_location_id) = input.emit(graph_builders, next_stmt_id).unwrap();

                graph_builders.entry(input_location_id).or_default()
                    .add_statement(parse_quote! {
                        #input_ident -> dest_sink(#sink);
                    });

                None
            }
            HfPlusNode::PipelineOp { pipeline, input } => {
                let (input_ident, input_location_id) = input.emit(graph_builders, next_stmt_id).unwrap();

                let pipeline_id = *next_stmt_id;
                *next_stmt_id += 1;

                let pipeline_ident = syn::Ident::new(&format!("stream_{}", pipeline_id), Span::call_site());

                graph_builders.entry(input_location_id).or_default()
                    .add_statement(parse_quote! {
                        #pipeline_ident = #input_ident -> #pipeline;
                    });

                Some((pipeline_ident, input_location_id))
            }
        }
    }

    pub fn build(&self) -> BTreeMap<usize, FlatGraphBuilder> {
        let mut out = BTreeMap::new();
        let mut next_id = 0;
        self.emit(&mut out, &mut next_id);

        out
    }
}

