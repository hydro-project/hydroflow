use quote::quote_spanned;
use syn::parse_quote_spanned;
use syn::spanned::Spanned;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OpInstGenerics, OperatorCategory,
    OperatorConstraints, OperatorInstance, WriteContextArgs, LATTICE_FOLD_REDUCE_FLOW_PROP_FN,
    RANGE_1,
};
use crate::graph::ops::OperatorWriteOutput;

/// > 1 input stream, 1 output stream
///
/// > Generic parameters: A `Lattice` type, must implement [`Merge<Self>`](https://hydro-project.github.io/hydroflow/doc/lattices/trait.Merge.html)
/// type.
///
/// A specialized operator for merging lattices together into an accumulated value. Like [`reduce()`](#reduce)
/// but specialized for lattice types. `lattice_reduce::<MyLattice>()` is equivalent to `reduce(hydroflow::lattices::Merge::merge_owned)`.
///
/// `lattice_reduce` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
/// within the same tick. With `'static`, values will be remembered across ticks and will be
/// aggregated with pairs arriving in later ticks. When not explicitly specified persistence
/// defaults to `'tick`.
///
/// `lattice_reduce` is differentiated from `lattice_fold` in that `lattice_reduce` does not require the accumulating type to implement `Default`.
/// But it also means that the accumulating function inputs and the accumulating type must be the same.
///
/// ```hydroflow
/// source_iter([1,2,3,4,5])
///     -> map(hydroflow::lattices::Max::new)
///     -> lattice_reduce::<'static, hydroflow::lattices::Max<usize>>()
///     -> assert_eq([hydroflow::lattices::Max::new(5)]);
/// ```
pub const LATTICE_REDUCE: OperatorConstraints = OperatorConstraints {
    name: "lattice_reduce",
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
    flow_prop_fn: Some(LATTICE_FOLD_REDUCE_FLOW_PROP_FN),
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   inputs,
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

        assert_eq!(1, inputs.len());
        let input = &inputs[0];

        assert_eq!(1, type_args.len());
        let lat_type = &type_args[0];

        let arguments = parse_quote_spanned! {lat_type.span()=> // Uses `lat_type.span()`!
            #root::lattices::Merge::<#lat_type>::merge
        };
        let wc = WriteContextArgs {
            op_inst: &OperatorInstance {
                arguments,
                ..op_inst.clone()
            },
            ..wc.clone()
        };

        let OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        } = (super::reduce::REDUCE.write_fn)(&wc, diagnostics)?;
        let write_iterator = quote_spanned! {lat_type.span()=> // Uses `lat_type.span()`!
            let #input = {
                /// Improve errors with `#lat_type` trait bound.
                #[inline(always)]
                fn check_inputs<Lat>(
                    input: impl ::std::iter::Iterator<Item = Lat>
                ) -> impl ::std::iter::Iterator<Item = Lat>
                where
                    Lat: #root::lattices::Merge<Lat>,
                {
                    input
                }
                check_inputs::<#lat_type>(#input)
            };
            #write_iterator
        };
        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
