use syn::parse_quote_spanned;
use syn::spanned::Spanned;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OpInstGenerics, OperatorCategory,
    OperatorConstraints, OperatorInstance, WriteContextArgs, RANGE_1,
};

/// > 1 input stream, 1 output stream
///
/// > Generic parameters: A `Lattice` type, must implement [`Merge<Self>`](https://hydro-project.github.io/hydroflow/doc/lattices/trait.Merge.html)
/// type.
///
/// A specialized operator for merging lattices together into a accumulated value. Like [`fold()`](#fold)
/// but specialized for lattice types. `lattice_fold::<MyLattice>()` is equivalent to `fold(MyLattice::default(), hydroflow::lattices::Merge::merge_owned)`.
///
/// `lattice_fold` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
/// within the same tick. With `'static`, values will be remembered across ticks and will be
/// aggregated with pairs arriving in later ticks. When not explicitly specified persistence
/// defaults to `'static`.
///
/// `lattice_fold` is differentiated from `lattice_reduce` in that `lattice_fold` can accumulate into a different type from its input.
/// But it also means that the accumulating type must have a sensible default value.
///
/// ```hydroflow
/// source_iter([hydroflow::lattices::set_union::SetUnionSingletonSet::new_from(7)])
///     -> lattice_fold::<'static, hydroflow::lattices::set_union::SetUnionHashSet<usize>>()
///     -> assert([hydroflow::lattices::set_union::SetUnionHashSet::new_from([7])]);
/// ```
pub const LATTICE_FOLD: OperatorConstraints = OperatorConstraints {
    name: "lattice_fold",
    categories: &[OperatorCategory::LatticeFold],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: &(0..=1),
    type_args: RANGE_1,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    properties: FlowProperties {
        deterministic: FlowPropertyVal::Preserve,
        monotonic: FlowPropertyVal::Yes,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| Some(DelayType::Stratum),
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   is_pull,
                   op_inst:
                       op_inst @ OperatorInstance {
                           generics: OpInstGenerics { type_args, .. },
                           ..
                       },
                   ..
               },
               diagnostics| {
        assert!(is_pull);

        let lat_type = &type_args[0];

        let arguments = parse_quote_spanned! {lat_type.span()=> // Uses `lat_type.span()`!
            <#lat_type>::default(), #root::lattices::Merge::merge_owned
        };

        let wc = WriteContextArgs {
            op_inst: &OperatorInstance {
                arguments,
                ..op_inst.clone()
            },
            ..wc.clone()
        };

        (super::fold::FOLD.write_fn)(&wc, diagnostics)
    },
};
