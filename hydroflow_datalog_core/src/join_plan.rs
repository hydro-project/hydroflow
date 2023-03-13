use std::collections::{btree_map::Entry, BTreeMap, HashMap};

use hydroflow_lang::graph::FlatGraphBuilder;
use hydroflow_lang::parse::Pipeline;
use proc_macro2::Span;
use syn::{self, parse_quote};

use crate::{
    grammar::datalog::{BoolOp, InputRelationExpr, PredicateExpr},
    util::{repeat_tuple, Counter},
};

/// Captures the tree of joins used to compute contributions from a single rule.
pub enum JoinPlan<'a> {
    /// A single relation without any joins, leaves of the tree.
    Source(&'a InputRelationExpr),
    /// A join between two subtrees.
    Join(Box<JoinPlan<'a>>, Box<JoinPlan<'a>>),
    AntiJoin(Box<JoinPlan<'a>>, Box<JoinPlan<'a>>),
    Predicate(Vec<&'a PredicateExpr>, Box<JoinPlan<'a>>),
}

/// Tracks the Hydroflow node that corresponds to a subtree of a join plan.
pub struct IntermediateJoinNode {
    /// The name of the Hydroflow node that this join outputs to.
    pub name: syn::Ident,
    /// If this join node outputs data through a `tee()` operator, this is the index to consume the node with.
    /// (this is only used for cases where we are directly reading a relation)
    pub tee_idx: Option<isize>,
    /// A mapping from variables in the rule to the index of the corresponding element in the flattened tuples this node emits.
    pub variable_mapping: BTreeMap<syn::Ident, usize>,
    /// The type of the flattened tuples this node emits.
    pub tuple_type: syn::Type,
}

enum JoinSide {
    Left,
    Right,
}

impl JoinSide {
    fn index(&self) -> usize {
        match self {
            JoinSide::Left => 0,
            JoinSide::Right => 1,
        }
    }
}

/// Generates a Hydroflow pipeline that transforms some input to a join
/// to emit key-value tuples that can be fed into a join operator.
fn emit_join_input_pipeline(
    // The identifiers of the input node that the key should be populated with.
    identifiers_to_join: &[&syn::Ident],
    // The Hydroflow node that is one side of the join.
    source_expanded: &IntermediateJoinNode,
    // The Hydroflow node for the join operator.
    join_node: &syn::Ident,
    // Whether this node contributes to the left or right side of the join.
    join_side: JoinSide,
    // Whether the pipeline is for an anti-join.
    anti_join: bool,
    // The Hydroflow graph to emit the pipeline to.
    flat_graph_builder: &mut FlatGraphBuilder,
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

    let out_index = syn::Index::from(join_side.index());

    let source_name = &source_expanded.name;
    let source_type = &source_expanded.tuple_type;

    let rhs: Pipeline = if anti_join {
        match join_side {
            JoinSide::Left => {
                parse_quote!(map(|v: #source_type| ((#(#hash_keys, )*), v)) -> [pos] #join_node)
            }
            JoinSide::Right => {
                parse_quote!(map(|v: #source_type| (#(#hash_keys, )*)) -> [neg] #join_node)
            }
        }
    } else {
        parse_quote!(map(|v: #source_type| ((#(#hash_keys, )*), v)) -> [#out_index] #join_node)
    };

    let statement = match source_expanded.tee_idx {
        Some(i) => {
            let in_index = syn::LitInt::new(&format!("{}", i), Span::call_site());
            parse_quote!(#source_name [#in_index] -> #rhs)
        }
        None => parse_quote!(#source_name -> #rhs),
    };

    flat_graph_builder.add_statement(statement);
}

/// Creates a mapping from variable names to the indices where that variable appears in `fields`.
///
/// Only return entries for variables that appear more than once. Those correspond to additional
/// constraints: the relation is only true when the values at those indices are equal.
///
/// For example, `rel(a, b, a) := ...` requires that the values in the 0th and 2nd slots be the
/// same, so we would return a map `{ "a" => [0, 2] }`. Note that since `b` is not repeated, it is
/// not in the map.
fn find_relation_local_constraints(
    fields: &[crate::grammar::datalog::Ident],
) -> BTreeMap<String, Vec<usize>> {
    let mut indices_grouped_by_var = BTreeMap::new();
    for (i, ident) in fields.iter().enumerate() {
        let entry = indices_grouped_by_var
            // TODO(shadaj): Can we avoid cloning here?
            .entry(ident.name.clone())
            .or_insert_with(Vec::new);
        entry.push(i);
    }

    indices_grouped_by_var.retain(|_, v| v.len() > 1);

    indices_grouped_by_var
}

/// Given a mapping from variable names to their repeated indices, builds a Rust expression that
/// tests whether the values at those indices are equal for each variable.
///
/// For example, `rel(a, b, a, a, b)` would give us the map `{ "a" => [0, 2, 3], "b" => [1, 4] }`.
/// Then we would want to generate the code `row.0 == row.2 && row.0 == row.3 && row.1 == row.4`.
fn build_local_constraint_conditions(constraints: &BTreeMap<String, Vec<usize>>) -> syn::Expr {
    constraints
        .values()
        .flat_map(|indices| {
            let equal_indices = indices
                .iter()
                .map(|i| syn::Index::from(*i))
                .collect::<Vec<_>>();

            let first_index = &equal_indices[0];

            equal_indices
                .iter()
                .skip(1)
                .map(|i| parse_quote!(row.#first_index == row.#i))
                .collect::<Vec<_>>()
        })
        .reduce(|a: syn::Expr, b| parse_quote!(#a && #b))
        .unwrap()
}

/// Generates a Hydroflow pipeline that computes the output to a given [`JoinPlan`].
pub fn expand_join_plan(
    // The plan we are converting to a Hydroflow pipeline.
    plan: &JoinPlan,
    // The Hydroflow graph to emit the pipeline to.
    flat_graph_builder: &mut FlatGraphBuilder,
    tee_counter: &mut HashMap<String, Counter>,
    next_join_idx: &mut Counter,
) -> IntermediateJoinNode {
    match plan {
        JoinPlan::Source(target) => {
            let mut variable_mapping = BTreeMap::new();
            let mut row_types: Vec<syn::Type> = vec![];

            let local_constraints = find_relation_local_constraints(&target.fields);

            for (i, ident) in target.fields.iter().enumerate() {
                row_types.push(parse_quote!(_));

                let variable_ident = syn::Ident::new(&ident.name, Span::call_site());
                if let Entry::Vacant(e) = variable_mapping.entry(variable_ident) {
                    e.insert(i);
                }
            }

            // Because this is a node corresponding to some Datalog relation, we need to tee from it.
            let tee_index = tee_counter
                .entry(target.name.name.clone())
                .or_insert_with(|| 0..)
                .next()
                .expect("Out of tee indices");

            let row_type = parse_quote!((#(#row_types, )*));

            if local_constraints.is_empty() {
                return IntermediateJoinNode {
                    name: syn::Ident::new(&target.name.name, Span::call_site()),
                    tee_idx: Some(tee_index),
                    variable_mapping,
                    tuple_type: row_type,
                };
            }

            let relation_node = syn::Ident::new(&target.name.name, Span::call_site());
            let relation_idx = syn::LitInt::new(&tee_index.to_string(), Span::call_site());

            let filter_node = syn::Ident::new(
                &format!(
                    "join_{}_filter",
                    next_join_idx.next().expect("Out of join indices")
                ),
                Span::call_site(),
            );

            let conditions = build_local_constraint_conditions(&local_constraints);

            flat_graph_builder.add_statement(parse_quote! {
                #filter_node = #relation_node [#relation_idx] -> filter(|&row: &#row_type| #conditions)
            });

            IntermediateJoinNode {
                name: filter_node,
                tee_idx: None,
                variable_mapping,
                tuple_type: row_type,
            }
        }
        JoinPlan::Join(lhs, rhs) | JoinPlan::AntiJoin(lhs, rhs) => {
            let is_anti = matches!(plan, JoinPlan::AntiJoin(_, _));

            let left_expanded =
                expand_join_plan(lhs, flat_graph_builder, tee_counter, next_join_idx);
            let right_expanded =
                expand_join_plan(rhs, flat_graph_builder, tee_counter, next_join_idx);

            let identifiers_to_join = right_expanded
                .variable_mapping
                .keys()
                .filter(|i| left_expanded.variable_mapping.contains_key(i))
                .collect::<Vec<_>>();

            let key_type =
                repeat_tuple::<syn::Type, syn::Type>(|| parse_quote!(_), identifiers_to_join.len());

            let left_type = &left_expanded.tuple_type;
            let right_type = &right_expanded.tuple_type;

            let join_node = syn::Ident::new(
                &format!(
                    "join_{}",
                    next_join_idx.next().expect("Out of join indices")
                ),
                Span::call_site(),
            );

            let intermediate = if is_anti {
                let flatten_closure: syn::Expr = parse_quote!(|kv: (#key_type, #left_type)| kv.1);

                flat_graph_builder
                    .add_statement(parse_quote!(#join_node = anti_join() -> map(#flatten_closure)));

                IntermediateJoinNode {
                    name: join_node.clone(),
                    tee_idx: None,
                    variable_mapping: left_expanded.variable_mapping.clone(),
                    tuple_type: left_expanded.tuple_type.clone(),
                }
            } else {
                // We start by defining the pipeline from the `join()` operator onwards. The main logic
                // here is to flatten the tuples from the left and right sides of the join into a
                // single tuple that is used by downstream joins or the final output.
                let mut flattened_tuple_elems: Vec<syn::Expr> = vec![];
                let mut flattened_mapping = BTreeMap::new();

                for (ident, source_idx) in left_expanded
                    .variable_mapping
                    .keys()
                    .map(|l| (l, 0))
                    .chain(right_expanded.variable_mapping.keys().map(|l| (l, 1)))
                {
                    if !flattened_mapping.contains_key(ident) {
                        let syn_source_index = syn::Index::from(source_idx);
                        let source_expr: syn::Expr = parse_quote!(kv.1.#syn_source_index);
                        let bindings = if source_idx == 0 {
                            &left_expanded.variable_mapping
                        } else {
                            &right_expanded.variable_mapping
                        };

                        let source_col_idx = syn::Index::from(*bindings.get(ident).unwrap());

                        flattened_mapping.insert(ident.clone(), flattened_tuple_elems.len());
                        flattened_tuple_elems.push(parse_quote!(#source_expr.#source_col_idx));
                    }
                }

                let flatten_closure: syn::Expr = parse_quote!(|kv: (#key_type, (#left_type, #right_type))| (#(#flattened_tuple_elems, )*));

                flat_graph_builder.add_statement(
                    parse_quote!(#join_node = join::<'tick>() -> map(#flatten_closure)),
                );

                let output_type = repeat_tuple::<syn::Type, syn::Type>(
                    || parse_quote!(_),
                    flattened_tuple_elems.len(),
                );

                IntermediateJoinNode {
                    name: join_node.clone(),
                    tee_idx: None,
                    variable_mapping: flattened_mapping,
                    tuple_type: output_type,
                }
            };

            emit_join_input_pipeline(
                &identifiers_to_join,
                &left_expanded,
                &join_node,
                JoinSide::Left,
                is_anti,
                flat_graph_builder,
            );

            emit_join_input_pipeline(
                &identifiers_to_join,
                &right_expanded,
                &join_node,
                JoinSide::Right,
                is_anti,
                flat_graph_builder,
            );

            intermediate
        }
        JoinPlan::Predicate(predicates, inner) => {
            let inner_expanded =
                expand_join_plan(inner, flat_graph_builder, tee_counter, next_join_idx);
            let inner_name = inner_expanded.name.clone();
            let row_type = inner_expanded.tuple_type;
            let variable_mapping = &inner_expanded.variable_mapping;

            let conditions = predicates
                .iter()
                .map(|p| {
                    let l_ident = syn::Ident::new(&p.left.name, Span::call_site());
                    let r_ident = syn::Ident::new(&p.right.name, Span::call_site());
                    let l = syn::Index::from(*variable_mapping.get(&l_ident).unwrap());
                    let r = syn::Index::from(*variable_mapping.get(&r_ident).unwrap());
                    match &p.op {
                        BoolOp::Lt(_) => parse_quote!(row.#l < row.#r),
                        BoolOp::LtEq(_) => parse_quote!(row.#l <= row.#r),
                        BoolOp::Gt(_) => parse_quote!(row.#l > row.#r),
                        BoolOp::GtEq(_) => parse_quote!(row.#l >= row.#r),
                        BoolOp::Eq(_) => parse_quote!(row.#l == row.#r),
                    }
                })
                .reduce(|a: syn::Expr, b| parse_quote!(#a && #b))
                .unwrap();

            let predicate_filter_node = syn::Ident::new(
                &format!(
                    "predicate_{}_filter",
                    next_join_idx.next().expect("Out of join indices")
                ),
                Span::call_site(),
            );

            flat_graph_builder.add_statement(parse_quote! {
                #predicate_filter_node = #inner_name -> filter(|&row: &#row_type| #conditions )
            });

            IntermediateJoinNode {
                name: predicate_filter_node,
                tee_idx: None,
                variable_mapping: inner_expanded.variable_mapping,
                tuple_type: row_type,
            }
        }
    }
}
