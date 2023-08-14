use syn::parse_quote;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints,
    WriteContextArgs, RANGE_1,
};
use crate::graph::{OpInstGenerics, OperatorInstance};

/// > 2 input streams of type `(K, V1)` and `(K, V2)`, 1 output stream of type `(K, (V1', V2'))` where `V1`, `V2`, `V1'`, `V2'` are lattice types
///
/// Performs a [`fold_keyed`](#fold_keyed) with lattice-merge aggregate function on each input and then forms the
/// equijoin of the resulting key/value pairs in the input streams by their first (key) attribute. Like [`join`](#join), the result nests the 2nd input field (values) into a tuple in the 2nd output field.
///
/// You must specify the the accumulating lattice types, they cannot be inferred. The first type argument corresponds to the `[0]` input of the join, and the second to the `[1]` input.
/// Type arguments are specified in hydroflow using the rust turbofish syntax `::<>`, for example `lattice_join::<Min<_>, Max<_>>()`
/// The accumulating lattice type is not necessarily the same type as the input, see the below example involving SetUnion for such a case.
///
/// Like [`join`](#join), `lattice_join` can also be provided with one or two generic lifetime persistence arguments, either
/// `'tick` or `'static`, to specify how join data persists. With `'tick`, pairs will only be
/// joined with corresponding pairs within the same tick. With `'static`, pairs will be remembered
/// across ticks and will be joined with pairs arriving in later ticks. When not explicitly
/// specified persistence defaults to `tick.
///
/// Like [`join`](#join), when two persistence arguments are supplied the first maps to port `0` and the second maps to
/// port `1`.
/// When a single persistence argument is supplied, it is applied to both input ports.
/// When no persistence arguments are applied it defaults to `'tick` for both.
/// It is important to specify all persistence arguments before any type arguments, otherwise the persistence arguments will be ignored.
///
/// The syntax is as follows:
/// ```hydroflow,ignore
/// lattice_join::<MaxRepr<usize>, MaxRepr<usize>>(); // Or
/// lattice_join::<'static, MaxRepr<usize>, MaxRepr<usize>>();
///
/// lattice_join::<'tick, MaxRepr<usize>, MaxRepr<usize>>();
///
/// lattice_join::<'static, 'tick, MaxRepr<usize>, MaxRepr<usize>>();
///
/// lattice_join::<'tick, 'static, MaxRepr<usize>, MaxRepr<usize>>();
/// // etc.
/// ```
///
/// ### Examples
///
/// ```hydroflow
/// use hydroflow::lattices::Min;
/// use hydroflow::lattices::Max;
///
/// source_iter([("key", Min::new(1)), ("key", Min::new(2))]) -> [0]my_join;
/// source_iter([("key", Max::new(1)), ("key", Max::new(2))]) -> [1]my_join;
///
/// my_join = lattice_join::<'tick, Min<usize>, Max<usize>>()
///     -> assert_eq([("key", (Min::new(1), Max::new(2)))]);
/// ```
///
/// ```hydroflow
/// use hydroflow::lattices::set_union::SetUnionSingletonSet;
/// use hydroflow::lattices::set_union::SetUnionHashSet;
///
/// source_iter([("key", SetUnionSingletonSet::new_from(0)), ("key", SetUnionSingletonSet::new_from(1))]) -> [0]my_join;
/// source_iter([("key", SetUnionHashSet::new_from([0])), ("key", SetUnionHashSet::new_from([1]))]) -> [1]my_join;
///
/// my_join = lattice_join::<'tick, SetUnionHashSet<usize>, SetUnionHashSet<usize>>()
///     -> assert_eq([("key", (SetUnionHashSet::new_from([0, 1]), SetUnionHashSet::new_from([0, 1])))]);
/// ```
pub const LATTICE_JOIN: OperatorConstraints = OperatorConstraints {
    name: "lattice_join",
    categories: &[OperatorCategory::MultiIn],
    hard_range_inn: &(2..=2),
    soft_range_inn: &(2..=2),
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: &(0..=2),
    type_args: &(2..=2),
    is_external_input: false,
    ports_inn: Some(|| super::PortListSpec::Fixed(parse_quote! { 0, 1 })),
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Preserve,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   op_inst:
                       op_inst @ OperatorInstance {
                           generics: OpInstGenerics { type_args, .. },
                           ..
                       },
                   ..
               },
               diagnostics| {
        let lhs_type = &type_args[0];
        let rhs_type = &type_args[1];

        let wc = WriteContextArgs {
            op_inst: &OperatorInstance {
                arguments: parse_quote! {
                    FoldFrom(<#lhs_type as #root::lattices::LatticeFrom::<_>>::lattice_from, #root::lattices::Merge::merge),
                    FoldFrom(<#rhs_type as #root::lattices::LatticeFrom::<_>>::lattice_from, #root::lattices::Merge::merge)
                },
                ..op_inst.clone()
            },
            ..wc.clone()
        };

        (super::join_fused::JOIN_FUSED.write_fn)(&wc, diagnostics)
    },
};
