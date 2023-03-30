use crate::graph::ops::OperatorWriteOutput;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OpInstGenerics, OperatorConstraints,
    OperatorInstance, WriteContextArgs, RANGE_1,
};

use quote::quote_spanned;
use syn::{parse_quote_spanned, spanned::Spanned};

/// > 1 input stream, 1 output stream
///
/// > Generic parameters: A [`LatticeRepr`](https://hydro-project.github.io/hydroflow/doc/hydroflow/lang/lattice/trait.LatticeRepr.html)
/// type.
///
/// A specialized operator for merging lattices together into a accumulated value. Like [`fold()`](#fold)
/// but specialized for lattice types. `lattice_merge::<MyLatRepr>()` is equivalent to `fold(Default::default, <MyLatRepr as LatticeRepr>::merge_owned)`.
///
/// `lattice_merge` can also be provided with one generic lifetime persistence argument, either
/// `'tick` or `'static`, to specify how data persists. With `'tick`, values will only be collected
/// within the same tick. With `'static`, values will be remembered across ticks and will be
/// aggregated with pairs arriving in later ticks. When not explicitly specified persistence
/// defaults to `'static`.
///
/// ```hydroflow
/// source_iter([1,2,3,4,5])
///     -> lattice_merge::<'static, hydroflow::lang::lattice::ord::MaxRepr<usize>>()
///     -> for_each(|x| println!("Least upper bound: {}", x));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const LATTICE_MERGE: OperatorConstraints = OperatorConstraints {
    name: "lattice_merge",
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
        let lat_repr = &type_args[0];

        let arguments = parse_quote_spanned! {lat_repr.span()=> // Uses `lat_repr.span()`!
            ::std::default::Default::default(), <#lat_repr as #root::lang::lattice::Merge<#lat_repr>>::merge_owned
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
        } = (super::fold::FOLD.write_fn)(&wc, diagnostics)?;
        let write_iterator = quote_spanned! {lat_repr.span()=> // Uses `lat_repr.span()`!
            let #input = {
                /// Improve errors with `#lat_repr` trait bound.
                #[inline(always)]
                fn check_inputs<Lr>(
                    input: impl Iterator<Item = <Lr as #root::lang::lattice::LatticeRepr>::Repr>
                ) -> impl Iterator<Item = <Lr as #root::lang::lattice::LatticeRepr>::Repr>
                where
                    Lr: #root::lang::lattice::LatticeRepr,
                {
                    input
                }
                check_inputs::<#lat_repr>(#input)
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
