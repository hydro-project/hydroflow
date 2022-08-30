use std::collections::{BTreeMap, HashMap};

use hydroflow_lang::{
    graph::flat_graph::FlatGraph,
    parse::{ArrowConnector, IndexInt, Indexing, Pipeline, PipelineLink},
};
use proc_macro2::Span;
use syn::{self, parse_quote};

use crate::grammar::datalog::Target;

pub enum JoinPlan {
    Source(usize),
    Join(Box<JoinPlan>, Box<JoinPlan>),
}

pub struct IntermediateJoinNode {
    /// The name of the Hydroflow node that this join outputs to.
    pub name: syn::Ident,
    /// If this join node outputs data through a `tee()` operator, this is the index to consume the node with.
    /// (this is only used for cases where we are directly reading a relation)
    pub tee_idx: Option<usize>,
    /// A mapping from variables in the rule to the index of the corresponding element in the flattened tuples this node emits.
    pub variable_mapping: BTreeMap<syn::Ident, usize>,
    /// The type of the flattened tuples this node emits.
    pub tuple_type: syn::Type,
}

fn emit_source_to_join(
    identifiers_to_join: &[&syn::Ident],
    source_expanded: &IntermediateJoinNode,
    output: (&syn::Ident, usize),
    flat_graph: &mut FlatGraph,
) {
    let hash_keys: Vec<syn::Expr> = identifiers_to_join
        .iter()
        .map(|ident| {
            if let Some(idx) = source_expanded.variable_mapping.get(ident) {
                let idx_ident = syn::Index::from(*idx);
                parse_quote!(v.#idx_ident)
            } else {
                panic!("Could not find key that is being joined on: {:?}", ident);
            }
        })
        .collect();

    let (out_node, out_idx) = output;
    let out_index = syn::Index::from(out_idx);

    let source_name = &source_expanded.name;
    let source_type = &source_expanded.tuple_type;
    flat_graph.add_statement(hydroflow_lang::parse::HfStatement::Pipeline(
        Pipeline::Link(PipelineLink {
            lhs: Box::new(parse_quote!(#source_name)),
            connector: ArrowConnector {
                src: source_expanded.tee_idx.map(|i| Indexing {
                    bracket_token: syn::token::Bracket::default(),
                    index: IndexInt {
                        value: i,
                        span: Span::call_site(),
                    },
                }),
                arrow: parse_quote!(->),
                dst: None,
            },
            rhs: Box::new(parse_quote! {
                map(|v: #source_type| ((#(#hash_keys, )*), v)) -> [#out_index] #out_node
            }),
        }),
    ));
}

// outputs the identifier for the join node and a mapping from rule identifiers to indices in the join output tuple
pub fn expand_join_plan(
    plan: &JoinPlan,
    all_sources: &[Target],
    flat_graph: &mut FlatGraph,
    tee_counter: &mut HashMap<String, usize>,
    merge_counter: &mut HashMap<String, usize>,
    next_join_idx: &mut usize,
) -> IntermediateJoinNode {
    match plan {
        JoinPlan::Source(idx) => {
            let target = &all_sources[*idx];
            let mut variable_mapping = BTreeMap::new();
            let mut row_types: Vec<syn::Type> = vec![];
            for (i, ident) in target.fields.iter().enumerate() {
                row_types.push(parse_quote!(_));
                let variable_ident = syn::Ident::new(&ident.name, Span::call_site());
                if let std::collections::btree_map::Entry::Vacant(e) =
                    variable_mapping.entry(variable_ident)
                {
                    e.insert(i);
                } else {
                    // TODO(shadaj): if there is already an entry in mapping that means filter
                    panic!()
                }
            }

            let tee_index = tee_counter.entry(target.name.name.clone()).or_insert(0);
            let my_tee_index = *tee_index;
            *tee_index += 1;

            IntermediateJoinNode {
                name: syn::Ident::new(&target.name.name, Span::call_site()),
                tee_idx: Some(my_tee_index),
                variable_mapping,
                tuple_type: parse_quote!((#(#row_types, )*)),
            }
        }
        JoinPlan::Join(lhs, rhs) => {
            let left_expanded = expand_join_plan(
                lhs,
                all_sources,
                flat_graph,
                tee_counter,
                merge_counter,
                next_join_idx,
            );

            let right_expanded = expand_join_plan(
                rhs,
                all_sources,
                flat_graph,
                tee_counter,
                merge_counter,
                next_join_idx,
            );

            let my_idx = *next_join_idx;
            *next_join_idx += 1;

            let identifiers_to_join = right_expanded
                .variable_mapping
                .keys()
                .filter(|i| left_expanded.variable_mapping.contains_key(i))
                .collect::<Vec<_>>();

            let mut output_data: Vec<syn::Expr> = vec![];
            let mut ident_to_index = BTreeMap::new();

            for (ident, source_idx) in left_expanded
                .variable_mapping
                .keys()
                .map(|l| (l, 0))
                .chain(right_expanded.variable_mapping.keys().map(|l| (l, 1)))
            {
                if !ident_to_index.contains_key(ident) {
                    let syn_source_index = syn::Index::from(source_idx);
                    let source_expr: syn::Expr = parse_quote!(kv.1.#syn_source_index);
                    let bindings = if source_idx == 0 {
                        &left_expanded.variable_mapping
                    } else {
                        &right_expanded.variable_mapping
                    };

                    let source_col_idx = syn::Index::from(*bindings.get(ident).unwrap());

                    ident_to_index.insert(ident.clone(), output_data.len());
                    output_data.push(parse_quote!(#source_expr.#source_col_idx));
                }
            }

            let key_type = identifiers_to_join
                .iter()
                .map(|_| parse_quote!(_))
                .collect::<Vec<syn::Type>>();

            let left_type = &left_expanded.tuple_type;
            let right_type = &right_expanded.tuple_type;
            let after_join_map: syn::Expr = parse_quote!(|kv: ((#(#key_type, )*), (#left_type, #right_type))| (#(#output_data, )*));

            let join_node = syn::Ident::new(&format!("join_{}", my_idx), Span::call_site());
            flat_graph.add_statement(parse_quote!(#join_node = join() -> map(#after_join_map)));

            emit_source_to_join(
                &identifiers_to_join,
                &left_expanded,
                (&join_node, 0),
                flat_graph,
            );

            emit_source_to_join(
                &identifiers_to_join,
                &right_expanded,
                (&join_node, 1),
                flat_graph,
            );

            let output_types: Vec<syn::Type> = output_data
                .iter()
                .map(|_| parse_quote!(_))
                .collect::<Vec<_>>();

            IntermediateJoinNode {
                name: join_node,
                tee_idx: None,
                variable_mapping: ident_to_index,
                tuple_type: parse_quote!((#(#output_types, )*)),
            }
        }
    }
}
