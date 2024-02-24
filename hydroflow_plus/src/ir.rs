use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::rc::Rc;

use hydroflow_lang::graph::FlatGraphBuilder;
use hydroflow_lang::parse::Pipeline;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse_quote;

#[derive(Clone)]
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

#[derive(Clone, Debug)]
pub enum HfPlusSource {
    Stream(DebugExpr),
    Iter(DebugExpr),
}

#[derive(Clone, Debug)]
pub enum HfPlusNode {
    Todo,
    Persist(Box<HfPlusNode>),
    Source {
        source: HfPlusSource,
        produces_delta: bool,
        location_id: usize,
    },
    Tee {
        inner: Rc<RefCell<HfPlusNode>>,
    },
    CrossProduct(Box<HfPlusNode>, Box<HfPlusNode>),
    Join(Box<HfPlusNode>, Box<HfPlusNode>),
    Difference(Box<HfPlusNode>, Box<HfPlusNode>),
    AntiJoin(Box<HfPlusNode>, Box<HfPlusNode>),
    // TODO(shadaj): separate types for operators that don't return anything?
    ForEach {
        f: DebugExpr,
        input: Box<HfPlusNode>,
    },
    DestSink {
        sink: DebugExpr,
        input: Box<HfPlusNode>,
        send_delta: bool,
    },
    PipelineOp {
        pipeline: Pipeline,
        produces_delta: bool,
        input: Box<HfPlusNode>,
    },
}

impl HfPlusNode {
    pub fn emit(
        &self,
        graph_builders: &mut BTreeMap<usize, FlatGraphBuilder>,
        built_tees: &mut HashMap<*const RefCell<HfPlusNode>, (syn::Ident, usize, bool)>,
        next_stmt_id: &mut usize,
    ) -> Option<(syn::Ident, usize, bool)> {
        match self {
            HfPlusNode::Todo => {
                todo!()
            }

            HfPlusNode::Persist(inner) => {
                let (ident, location, inner_delta) = inner
                    .emit(graph_builders, built_tees, next_stmt_id)
                    .unwrap();
                assert!(!inner_delta, "double persist");
                Some((ident, location, true))
            }

            HfPlusNode::Source {
                source,
                produces_delta,
                location_id,
            } => {
                let source_id = *next_stmt_id;
                *next_stmt_id += 1;

                let source_ident =
                    syn::Ident::new(&format!("stream_{}", source_id), Span::call_site());

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

                graph_builders
                    .entry(*location_id)
                    .or_default()
                    .add_statement(source_stmt);

                Some((source_ident, *location_id, *produces_delta))
            }

            HfPlusNode::Tee { inner } => {
                if let Some(ret) = built_tees.get(&(inner.as_ref() as *const RefCell<HfPlusNode>)) {
                    Some(ret.clone())
                } else {
                    let (inner_ident, inner_location_id, inner_produces_delta) = inner
                        .borrow()
                        .emit(graph_builders, built_tees, next_stmt_id)
                        .unwrap();

                    let tee_id = *next_stmt_id;
                    *next_stmt_id += 1;

                    let tee_ident =
                        syn::Ident::new(&format!("stream_{}", tee_id), Span::call_site());

                    let builder = graph_builders.entry(inner_location_id).or_default();
                    builder.add_statement(parse_quote! {
                        #tee_ident = #inner_ident -> tee();
                    });

                    built_tees.insert(
                        inner.as_ref() as *const RefCell<HfPlusNode>,
                        (tee_ident.clone(), inner_location_id, inner_produces_delta),
                    );

                    Some((tee_ident, inner_location_id, inner_produces_delta))
                }
            }

            HfPlusNode::CrossProduct(left, right) => {
                let (left_ident, left_location_id, left_delta) =
                    left.emit(graph_builders, built_tees, next_stmt_id).unwrap();
                let (right_ident, right_location_id, right_delta) = right
                    .emit(graph_builders, built_tees, next_stmt_id)
                    .unwrap();

                assert_eq!(
                    left_location_id, right_location_id,
                    "cross product inputs must be in the same location"
                );

                let cross_product_id = *next_stmt_id;
                *next_stmt_id += 1;

                let cross_product_ident =
                    syn::Ident::new(&format!("stream_{}", cross_product_id), Span::call_site());

                let builder = graph_builders.entry(left_location_id).or_default();

                let output_delta = match (left_delta, right_delta) {
                    (true, true) => {
                        builder.add_statement(parse_quote! {
                            #cross_product_ident = cross_join::<'static, 'static>();
                        });

                        false // TODO(shadaj): cross_join already replays?
                    }
                    (true, false) => {
                        builder.add_statement(parse_quote! {
                            #cross_product_ident = cross_join::<'static, 'tick>();
                        });

                        false
                    }
                    (false, true) => {
                        builder.add_statement(parse_quote! {
                            #cross_product_ident = cross_join::<'tick, 'static>();
                        });

                        false
                    }
                    (false, false) => {
                        builder.add_statement(parse_quote! {
                            #cross_product_ident = cross_join::<'tick, 'tick>();
                        });

                        false
                    }
                };

                builder.add_statement(parse_quote! {
                    #left_ident -> [0]#cross_product_ident;
                });

                builder.add_statement(parse_quote! {
                    #right_ident -> [1]#cross_product_ident;
                });

                Some((cross_product_ident, left_location_id, output_delta))
            }

            HfPlusNode::Join(left, right) => {
                let (left_ident, left_location_id, left_delta) =
                    left.emit(graph_builders, built_tees, next_stmt_id).unwrap();
                let (right_ident, right_location_id, right_delta) = right
                    .emit(graph_builders, built_tees, next_stmt_id)
                    .unwrap();

                assert_eq!(
                    left_location_id, right_location_id,
                    "cross product inputs must be in the same location"
                );

                let cross_product_id = *next_stmt_id;
                *next_stmt_id += 1;

                let join_ident =
                    syn::Ident::new(&format!("stream_{}", cross_product_id), Span::call_site());

                let builder = graph_builders.entry(left_location_id).or_default();
                let output_delta = match (left_delta, right_delta) {
                    (true, true) => {
                        builder.add_statement(parse_quote! {
                            #join_ident = join::<'static, 'static>();
                        });

                        false // TODO(shadaj): join already replays?
                    }
                    (true, false) => {
                        builder.add_statement(parse_quote! {
                            #join_ident = join::<'static, 'tick>();
                        });

                        false
                    }
                    (false, true) => {
                        builder.add_statement(parse_quote! {
                            #join_ident = join::<'tick, 'static>();
                        });

                        false
                    }
                    (false, false) => {
                        builder.add_statement(parse_quote! {
                            #join_ident = join::<'tick, 'tick>();
                        });

                        false
                    }
                };

                builder.add_statement(parse_quote! {
                    #left_ident -> [0]#join_ident;
                });

                builder.add_statement(parse_quote! {
                    #right_ident -> [1]#join_ident;
                });

                Some((join_ident, left_location_id, output_delta))
            }

            HfPlusNode::Difference(left, right) => {
                let (left_ident, left_location_id, left_delta) =
                    left.emit(graph_builders, built_tees, next_stmt_id).unwrap();
                let (right_ident, right_location_id, right_delta) = right
                    .emit(graph_builders, built_tees, next_stmt_id)
                    .unwrap();

                assert_eq!(
                    left_location_id, right_location_id,
                    "difference inputs must be in the same location"
                );

                let difference_id = *next_stmt_id;
                *next_stmt_id += 1;

                let difference_ident =
                    syn::Ident::new(&format!("stream_{}", difference_id), Span::call_site());

                let builder = graph_builders.entry(left_location_id).or_default();
                let output_delta = match (left_delta, right_delta) {
                    (true, true) => {
                        // difference/anti_join<'static, _> does not replay
                        // but we need to re-filter every tick
                        builder.add_statement(parse_quote! {
                            #difference_ident = difference::<'tick, 'static>();
                        });

                        false
                    }
                    (true, false) => {
                        // difference/anti_join<'static, _> does not replay
                        // but we need to re-filter every tick
                        builder.add_statement(parse_quote! {
                            #difference_ident = difference::<'tick, 'tick>();
                        });

                        false
                    }
                    (false, true) => {
                        builder.add_statement(parse_quote! {
                            #difference_ident = difference::<'tick, 'static>();
                        });

                        false
                    }
                    (false, false) => {
                        builder.add_statement(parse_quote! {
                            #difference_ident = difference::<'tick, 'tick>();
                        });

                        false
                    }
                };

                if left_delta {
                    builder.add_statement(parse_quote! {
                        #left_ident -> persist() -> [pos]#difference_ident;
                    });
                } else {
                    builder.add_statement(parse_quote! {
                        #left_ident -> [pos]#difference_ident;
                    });
                }

                builder.add_statement(parse_quote! {
                    #right_ident -> [neg]#difference_ident;
                });

                Some((difference_ident, left_location_id, output_delta))
            }

            HfPlusNode::AntiJoin(left, right) => {
                let (left_ident, left_location_id, left_delta) =
                    left.emit(graph_builders, built_tees, next_stmt_id).unwrap();
                let (right_ident, right_location_id, right_delta) = right
                    .emit(graph_builders, built_tees, next_stmt_id)
                    .unwrap();

                assert_eq!(
                    left_location_id, right_location_id,
                    "anti_join inputs must be in the same location"
                );

                let anti_join_id = *next_stmt_id;
                *next_stmt_id += 1;

                let anti_join_ident =
                    syn::Ident::new(&format!("stream_{}", anti_join_id), Span::call_site());

                let builder = graph_builders.entry(left_location_id).or_default();
                let output_delta = match (left_delta, right_delta) {
                    (true, true) => {
                        // difference/anti_join<'static, _> does not replay
                        // but we need to re-filter every tick
                        builder.add_statement(parse_quote! {
                            #anti_join_ident = anti_join::<'tick, 'static>();
                        });

                        false
                    }
                    (true, false) => {
                        // difference/anti_join<'static, _> does not replay
                        // but we need to re-filter every tick
                        builder.add_statement(parse_quote! {
                            #anti_join_ident = anti_join::<'tick, 'tick>();
                        });

                        false
                    }
                    (false, true) => {
                        builder.add_statement(parse_quote! {
                            #anti_join_ident = anti_join::<'tick, 'static>();
                        });

                        false
                    }
                    (false, false) => {
                        builder.add_statement(parse_quote! {
                            #anti_join_ident = anti_join::<'tick, 'tick>();
                        });

                        false
                    }
                };

                if left_delta {
                    builder.add_statement(parse_quote! {
                        #left_ident -> persist() -> [pos]#anti_join_ident;
                    });
                } else {
                    builder.add_statement(parse_quote! {
                        #left_ident -> [pos]#anti_join_ident;
                    });
                }

                builder.add_statement(parse_quote! {
                    #right_ident -> [neg]#anti_join_ident;
                });

                Some((anti_join_ident, left_location_id, output_delta))
            }

            HfPlusNode::ForEach { f, input } => {
                let (input_ident, input_location_id, input_delta) = input
                    .emit(graph_builders, built_tees, next_stmt_id)
                    .unwrap();

                assert!(!input_delta, "for_each on a delta stream");

                graph_builders
                    .entry(input_location_id)
                    .or_default()
                    .add_statement(parse_quote! {
                        #input_ident -> for_each(#f);
                    });

                None
            }

            HfPlusNode::DestSink {
                sink,
                input,
                send_delta,
            } => {
                let (input_ident, input_location_id, input_delta) = input
                    .emit(graph_builders, built_tees, next_stmt_id)
                    .unwrap();

                assert_eq!(input_delta, *send_delta, "delta mismatch in dest_sink");

                graph_builders
                    .entry(input_location_id)
                    .or_default()
                    .add_statement(parse_quote! {
                        #input_ident -> dest_sink(#sink);
                    });

                None
            }

            HfPlusNode::PipelineOp {
                pipeline,
                input,
                produces_delta,
            } => {
                let (input_ident, input_location_id, _input_delta) = input
                    .emit(graph_builders, built_tees, next_stmt_id)
                    .unwrap();

                let pipeline_id = *next_stmt_id;
                *next_stmt_id += 1;

                let pipeline_ident =
                    syn::Ident::new(&format!("stream_{}", pipeline_id), Span::call_site());

                graph_builders
                    .entry(input_location_id)
                    .or_default()
                    .add_statement(parse_quote! {
                        #pipeline_ident = #input_ident -> #pipeline;
                    });

                Some((pipeline_ident, input_location_id, *produces_delta))
            }
        }
    }

    pub fn build(&self) -> BTreeMap<usize, FlatGraphBuilder> {
        let mut out = BTreeMap::new();
        let mut next_id = 0;
        let mut built_tees = HashMap::new();
        self.emit(&mut out, &mut built_tees, &mut next_id);

        out
    }
}
