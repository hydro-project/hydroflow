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
        write!(f, "{}", self.0.to_token_stream())
    }
}

#[derive(Clone)]
pub struct DebugPipelineFn(pub Rc<dyn Fn() -> Pipeline + 'static>);

impl std::fmt::Debug for DebugPipelineFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<function>")
    }
}

/// A source in a Hydroflow+ graph, where data enters the graph.
#[derive(Clone, Debug)]
pub enum HfPlusSource {
    Stream(DebugExpr),
    Iter(DebugExpr),
    Interval(DebugExpr),
    Spin(),
}

/// An leaf in a Hydroflow+ graph, which is an pipeline that doesn't emit
/// any downstream values. Traversals over the dataflow graph and
/// generating Hydroflow IR start from leaves.
#[derive(Clone, Debug)]
pub enum HfPlusLeaf {
    ForEach {
        f: DebugExpr,
        input: Box<HfPlusNode>,
    },
    DestSink {
        sink: DebugExpr,
        input: Box<HfPlusNode>,
    },
    CycleSink {
        ident: syn::Ident,
        location_id: usize,
        input: Box<HfPlusNode>,
    },
}

impl HfPlusLeaf {
    pub fn transform_children(
        self,
        transform: impl Fn(HfPlusNode, &mut SeenTees) -> HfPlusNode,
        seen_tees: &mut SeenTees,
    ) -> HfPlusLeaf {
        match self {
            HfPlusLeaf::ForEach { f, input } => HfPlusLeaf::ForEach {
                f,
                input: Box::new(transform(*input, seen_tees)),
            },
            HfPlusLeaf::DestSink { sink, input } => HfPlusLeaf::DestSink {
                sink,
                input: Box::new(transform(*input, seen_tees)),
            },
            HfPlusLeaf::CycleSink {
                ident,
                location_id,
                input,
            } => HfPlusLeaf::CycleSink {
                ident,
                location_id,
                input: Box::new(transform(*input, seen_tees)),
            },
        }
    }

    pub fn emit(
        self,
        graph_builders: &mut BTreeMap<usize, FlatGraphBuilder>,
        built_tees: &mut HashMap<*const RefCell<HfPlusNode>, (syn::Ident, usize)>,
        next_stmt_id: &mut usize,
    ) {
        match self {
            HfPlusLeaf::ForEach { f, input } => {
                let (input_ident, input_location_id) =
                    input.emit(graph_builders, built_tees, next_stmt_id);

                graph_builders
                    .entry(input_location_id)
                    .or_default()
                    .add_statement(parse_quote! {
                        #input_ident -> for_each(#f);
                    });
            }

            HfPlusLeaf::DestSink { sink, input } => {
                let (input_ident, input_location_id) =
                    input.emit(graph_builders, built_tees, next_stmt_id);

                graph_builders
                    .entry(input_location_id)
                    .or_default()
                    .add_statement(parse_quote! {
                        #input_ident -> dest_sink(#sink);
                    });
            }

            HfPlusLeaf::CycleSink {
                ident,
                location_id,
                input,
            } => {
                let (input_ident, input_location_id) =
                    input.emit(graph_builders, built_tees, next_stmt_id);

                assert_eq!(
                    input_location_id, location_id,
                    "cycle_sink location mismatch"
                );

                graph_builders
                    .entry(location_id)
                    .or_default()
                    .add_statement(parse_quote! {
                        #ident = #input_ident;
                    });
            }
        }
    }
}

/// An intermediate node in a Hydroflow+ graph, which consumes data
/// from upstream nodes and emits data to downstream nodes.
#[derive(Clone, Debug)]
pub enum HfPlusNode {
    Placeholder,
    Persist(Box<HfPlusNode>),
    Delta(Box<HfPlusNode>),
    Source {
        source: HfPlusSource,
        location_id: usize,
    },
    CycleSource {
        ident: syn::Ident,
        location_id: usize,
    },
    Tee {
        inner: Rc<RefCell<HfPlusNode>>,
    },
    Union(Box<HfPlusNode>, Box<HfPlusNode>),
    CrossProduct(Box<HfPlusNode>, Box<HfPlusNode>),
    Join(Box<HfPlusNode>, Box<HfPlusNode>),
    Difference(Box<HfPlusNode>, Box<HfPlusNode>),
    AntiJoin(Box<HfPlusNode>, Box<HfPlusNode>),

    Map {
        f: DebugExpr,
        input: Box<HfPlusNode>,
    },

    Reduce {
        f: DebugExpr,
        input: Box<HfPlusNode>,
    },

    ReduceKeyed {
        f: DebugExpr,
        input: Box<HfPlusNode>,
    },

    /// A temporary IR node for a custom pipeline, which eventually will be replaced by a proper node.
    PipelineOp {
        kind: &'static str,
        gen_pipeline: DebugPipelineFn,
        input: Box<HfPlusNode>,
    },

    Network {
        to_location: usize,
        serialize_pipeline: Option<Pipeline>,
        sink_expr: DebugExpr,
        source_expr: DebugExpr,
        deserialize_pipeline: Option<Pipeline>,
        input: Box<HfPlusNode>,
    },
}

pub type SeenTees = HashMap<*const RefCell<HfPlusNode>, Rc<RefCell<HfPlusNode>>>;

impl HfPlusNode {
    pub fn transform_children(
        self,
        transform: impl Fn(HfPlusNode, &mut SeenTees) -> HfPlusNode,
        seen_tees: &mut SeenTees,
    ) -> HfPlusNode {
        match self {
            HfPlusNode::Placeholder => HfPlusNode::Placeholder,
            HfPlusNode::Persist(inner) => {
                HfPlusNode::Persist(Box::new(transform(*inner, seen_tees)))
            }
            HfPlusNode::Delta(inner) => HfPlusNode::Delta(Box::new(transform(*inner, seen_tees))),
            HfPlusNode::Source {
                source,
                location_id,
            } => HfPlusNode::Source {
                source,
                location_id,
            },
            HfPlusNode::CycleSource { ident, location_id } => {
                HfPlusNode::CycleSource { ident, location_id }
            }
            HfPlusNode::Tee { inner } => {
                if let Some(transformed) =
                    seen_tees.get(&(inner.as_ref() as *const RefCell<HfPlusNode>))
                {
                    HfPlusNode::Tee {
                        inner: transformed.clone(),
                    }
                } else {
                    let transformed_cell = Rc::new(RefCell::new(HfPlusNode::Placeholder));
                    seen_tees.insert(
                        inner.as_ref() as *const RefCell<HfPlusNode>,
                        transformed_cell.clone(),
                    );
                    let orig = inner.borrow().clone();
                    *transformed_cell.borrow_mut() = transform(orig, seen_tees);
                    HfPlusNode::Tee {
                        inner: transformed_cell,
                    }
                }
            }
            HfPlusNode::Union(left, right) => HfPlusNode::Union(
                Box::new(transform(*left, seen_tees)),
                Box::new(transform(*right, seen_tees)),
            ),
            HfPlusNode::CrossProduct(left, right) => HfPlusNode::CrossProduct(
                Box::new(transform(*left, seen_tees)),
                Box::new(transform(*right, seen_tees)),
            ),
            HfPlusNode::Join(left, right) => HfPlusNode::Join(
                Box::new(transform(*left, seen_tees)),
                Box::new(transform(*right, seen_tees)),
            ),
            HfPlusNode::Difference(left, right) => HfPlusNode::Difference(
                Box::new(transform(*left, seen_tees)),
                Box::new(transform(*right, seen_tees)),
            ),
            HfPlusNode::AntiJoin(left, right) => HfPlusNode::AntiJoin(
                Box::new(transform(*left, seen_tees)),
                Box::new(transform(*right, seen_tees)),
            ),
            HfPlusNode::Map { f, input } => HfPlusNode::Map {
                f,
                input: Box::new(transform(*input, seen_tees)),
            },
            HfPlusNode::Reduce { f, input } => HfPlusNode::Reduce {
                f,
                input: Box::new(transform(*input, seen_tees)),
            },
            HfPlusNode::ReduceKeyed { f, input } => HfPlusNode::ReduceKeyed {
                f,
                input: Box::new(transform(*input, seen_tees)),
            },
            HfPlusNode::PipelineOp {
                kind,
                gen_pipeline,
                input,
            } => HfPlusNode::PipelineOp {
                kind,
                gen_pipeline,
                input: Box::new(transform(*input, seen_tees)),
            },
            HfPlusNode::Network {
                to_location,
                serialize_pipeline,
                sink_expr,
                source_expr,
                deserialize_pipeline,
                input,
            } => HfPlusNode::Network {
                to_location,
                serialize_pipeline,
                sink_expr,
                source_expr,
                deserialize_pipeline,
                input: Box::new(transform(*input, seen_tees)),
            },
        }
    }

    pub fn emit(
        self,
        graph_builders: &mut BTreeMap<usize, FlatGraphBuilder>,
        built_tees: &mut HashMap<*const RefCell<HfPlusNode>, (syn::Ident, usize)>,
        next_stmt_id: &mut usize,
    ) -> (syn::Ident, usize) {
        match self {
            HfPlusNode::Placeholder => {
                panic!()
            }

            HfPlusNode::Persist(inner) => {
                let (inner_ident, location) = inner.emit(graph_builders, built_tees, next_stmt_id);

                let persist_id = *next_stmt_id;
                *next_stmt_id += 1;

                let persist_ident =
                    syn::Ident::new(&format!("stream_{}", persist_id), Span::call_site());

                let builder = graph_builders.entry(location).or_default();
                builder.add_statement(parse_quote! {
                    #persist_ident = #inner_ident -> persist();
                });

                (persist_ident, location)
            }

            HfPlusNode::Delta(inner) => {
                let (inner_ident, location) = inner.emit(graph_builders, built_tees, next_stmt_id);

                let delta_id = *next_stmt_id;
                *next_stmt_id += 1;

                let delta_ident =
                    syn::Ident::new(&format!("stream_{}", delta_id), Span::call_site());

                let builder = graph_builders.entry(location).or_default();
                builder.add_statement(parse_quote! {
                    #delta_ident = #inner_ident -> multiset_delta();
                });

                (delta_ident, location)
            }

            HfPlusNode::Source {
                source,
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

                    HfPlusSource::Interval(expr) => {
                        parse_quote! {
                            #source_ident = source_interval(#expr);
                        }
                    }

                    HfPlusSource::Spin() => {
                        parse_quote! {
                            #source_ident = spin();
                        }
                    }
                };

                graph_builders
                    .entry(location_id)
                    .or_default()
                    .add_statement(source_stmt);

                (source_ident, location_id)
            }

            HfPlusNode::CycleSource { ident, location_id } => (ident.clone(), location_id),

            HfPlusNode::Tee { inner } => {
                if let Some(ret) = built_tees.get(&(inner.as_ref() as *const RefCell<HfPlusNode>)) {
                    ret.clone()
                } else {
                    let (inner_ident, inner_location_id) = inner
                        .replace(HfPlusNode::Placeholder)
                        .emit(graph_builders, built_tees, next_stmt_id);

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
                        (tee_ident.clone(), inner_location_id),
                    );

                    (tee_ident, inner_location_id)
                }
            }

            HfPlusNode::Union(left, right) => {
                let (left_ident, left_location_id) =
                    left.emit(graph_builders, built_tees, next_stmt_id);
                let (right_ident, right_location_id) =
                    right.emit(graph_builders, built_tees, next_stmt_id);

                assert_eq!(
                    left_location_id, right_location_id,
                    "union inputs must be in the same location"
                );

                let union_id = *next_stmt_id;
                *next_stmt_id += 1;

                let union_ident =
                    syn::Ident::new(&format!("stream_{}", union_id), Span::call_site());

                let builder = graph_builders.entry(left_location_id).or_default();
                builder.add_statement(parse_quote! {
                    #union_ident = union();
                });

                builder.add_statement(parse_quote! {
                    #left_ident -> [0]#union_ident;
                });

                builder.add_statement(parse_quote! {
                    #right_ident -> [1]#union_ident;
                });

                (union_ident, left_location_id)
            }

            HfPlusNode::CrossProduct(..) | HfPlusNode::Join(..) => {
                let operator: syn::Ident = if matches!(self, HfPlusNode::CrossProduct(..)) {
                    parse_quote!(cross_join)
                } else {
                    parse_quote!(join)
                };

                if let HfPlusNode::CrossProduct(left, right) | HfPlusNode::Join(left, right) = self
                {
                    let (left, left_was_persist) = if let HfPlusNode::Persist(left) = *left {
                        (left, true)
                    } else {
                        (left, false)
                    };

                    let (right, right_was_persist) = if let HfPlusNode::Persist(right) = *right {
                        (right, true)
                    } else {
                        (right, false)
                    };

                    let (left_ident, left_location_id) =
                        left.emit(graph_builders, built_tees, next_stmt_id);
                    let (right_ident, right_location_id) =
                        right.emit(graph_builders, built_tees, next_stmt_id);

                    assert_eq!(
                        left_location_id, right_location_id,
                        "join / cross product inputs must be in the same location"
                    );

                    let stream_id = *next_stmt_id;
                    *next_stmt_id += 1;

                    let stream_ident =
                        syn::Ident::new(&format!("stream_{}", stream_id), Span::call_site());

                    let builder = graph_builders.entry(left_location_id).or_default();

                    match (left_was_persist, right_was_persist) {
                        (true, true) => {
                            builder.add_statement(parse_quote! {
                                #stream_ident = #operator::<'static, 'static>();
                            });
                        }
                        (true, false) => {
                            builder.add_statement(parse_quote! {
                                #stream_ident = #operator::<'static, 'tick>();
                            });
                        }
                        (false, true) => {
                            builder.add_statement(parse_quote! {
                                #stream_ident = #operator::<'tick, 'static>();
                            });
                        }
                        (false, false) => {
                            builder.add_statement(parse_quote! {
                                #stream_ident = #operator::<'tick, 'tick>();
                            });
                        }
                    };

                    builder.add_statement(parse_quote! {
                        #left_ident -> [0]#stream_ident;
                    });

                    builder.add_statement(parse_quote! {
                        #right_ident -> [1]#stream_ident;
                    });

                    (stream_ident, left_location_id)
                } else {
                    unreachable!()
                }
            }

            HfPlusNode::Difference(..) | HfPlusNode::AntiJoin(..) => {
                let operator: syn::Ident = if matches!(self, HfPlusNode::Difference(..)) {
                    parse_quote!(difference)
                } else {
                    parse_quote!(anti_join)
                };

                if let HfPlusNode::Difference(left, right) | HfPlusNode::AntiJoin(left, right) =
                    self
                {
                    let (right, right_was_persist) = if let HfPlusNode::Persist(right) = *right {
                        (right, true)
                    } else {
                        (right, false)
                    };

                    let (left_ident, left_location_id) =
                        left.emit(graph_builders, built_tees, next_stmt_id);
                    let (right_ident, right_location_id) =
                        right.emit(graph_builders, built_tees, next_stmt_id);

                    assert_eq!(
                        left_location_id, right_location_id,
                        "difference / anti join inputs must be in the same location"
                    );

                    let stream_id = *next_stmt_id;
                    *next_stmt_id += 1;

                    let stream_ident =
                        syn::Ident::new(&format!("stream_{}", stream_id), Span::call_site());

                    let builder = graph_builders.entry(left_location_id).or_default();

                    if right_was_persist {
                        builder.add_statement(parse_quote! {
                            #stream_ident = #operator::<'tick, 'static>();
                        });
                    } else {
                        builder.add_statement(parse_quote! {
                            #stream_ident = #operator::<'tick, 'tick>();
                        });
                    }

                    builder.add_statement(parse_quote! {
                        #left_ident -> [pos]#stream_ident;
                    });

                    builder.add_statement(parse_quote! {
                        #right_ident -> [neg]#stream_ident;
                    });

                    (stream_ident, left_location_id)
                } else {
                    unreachable!()
                }
            }

            HfPlusNode::Map { f, input } => {
                let (input_ident, input_location_id) =
                    input.emit(graph_builders, built_tees, next_stmt_id);

                let map_id = *next_stmt_id;
                *next_stmt_id += 1;

                let map_ident = syn::Ident::new(&format!("stream_{}", map_id), Span::call_site());

                let builder = graph_builders.entry(input_location_id).or_default();
                builder.add_statement(parse_quote! {
                    #map_ident = map(#f);
                });

                builder.add_statement(parse_quote! {
                    #input_ident -> #map_ident;
                });

                (map_ident, input_location_id)
            }

            HfPlusNode::Reduce { .. } | HfPlusNode::ReduceKeyed { .. } => {
                let operator: syn::Ident = if matches!(self, HfPlusNode::Reduce { .. }) {
                    parse_quote!(reduce)
                } else {
                    parse_quote!(reduce_keyed)
                };

                if let HfPlusNode::Reduce { f, input } | HfPlusNode::ReduceKeyed { f, input } = self
                {
                    let (input, input_was_persist) = if let HfPlusNode::Persist(input) = *input {
                        (input, true)
                    } else {
                        (input, false)
                    };

                    let (input_ident, input_location_id) =
                        input.emit(graph_builders, built_tees, next_stmt_id);

                    let reduce_id = *next_stmt_id;
                    *next_stmt_id += 1;

                    let reduce_ident =
                        syn::Ident::new(&format!("stream_{}", reduce_id), Span::call_site());

                    let builder = graph_builders.entry(input_location_id).or_default();
                    if input_was_persist {
                        builder.add_statement(parse_quote! {
                            #reduce_ident = #operator::<'static>(#f);
                        });
                    } else {
                        builder.add_statement(parse_quote! {
                            #reduce_ident = #operator::<'tick>(#f);
                        });
                    }

                    builder.add_statement(parse_quote! {
                        #input_ident -> #reduce_ident;
                    });

                    (reduce_ident, input_location_id)
                } else {
                    unreachable!()
                }
            }

            HfPlusNode::Network {
                to_location,
                serialize_pipeline,
                sink_expr,
                source_expr,
                deserialize_pipeline,
                input,
            } => {
                let (input_ident, input_location_id) =
                    input.emit(graph_builders, built_tees, next_stmt_id);

                let sender_builder = graph_builders.entry(input_location_id).or_default();

                if let Some(serialize_pipeline) = serialize_pipeline {
                    sender_builder.add_statement(parse_quote! {
                        #input_ident -> #serialize_pipeline -> dest_sink(#sink_expr);
                    });
                } else {
                    sender_builder.add_statement(parse_quote! {
                        #input_ident -> dest_sink(#sink_expr);
                    });
                }

                let receiver_builder = graph_builders.entry(to_location).or_default();
                let receiver_stream_id = *next_stmt_id;
                *next_stmt_id += 1;

                let receiver_stream_ident =
                    syn::Ident::new(&format!("stream_{}", receiver_stream_id), Span::call_site());

                if let Some(deserialize_pipeline) = deserialize_pipeline {
                    receiver_builder.add_statement(parse_quote! {
                        #receiver_stream_ident = source_stream(#source_expr) -> #deserialize_pipeline;
                    });
                } else {
                    receiver_builder.add_statement(parse_quote! {
                        #receiver_stream_ident = source_stream(#source_expr);
                    });
                }

                (receiver_stream_ident, to_location)
            }

            HfPlusNode::PipelineOp {
                kind: _,
                gen_pipeline,
                input,
            } => {
                let (input_ident, input_location_id) =
                    input.emit(graph_builders, built_tees, next_stmt_id);

                let pipeline_id = *next_stmt_id;
                *next_stmt_id += 1;

                let pipeline_ident =
                    syn::Ident::new(&format!("stream_{}", pipeline_id), Span::call_site());

                let pipeline = gen_pipeline.0();
                graph_builders
                    .entry(input_location_id)
                    .or_default()
                    .add_statement(parse_quote! {
                        #pipeline_ident = #input_ident -> #pipeline;
                    });

                (pipeline_ident, input_location_id)
            }
        }
    }
}
