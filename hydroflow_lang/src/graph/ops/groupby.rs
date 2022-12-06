use super::{
    DelayType, OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs,
    RANGE_1,
};

use quote::quote_spanned;

/// > 1 input stream of type (K,V1), 1 output stream of type (K,V2).
/// The output will have one tuple for each distinct K, with an accumulated value of type V2.
///
/// > Arguments: two Rust closures. The first generates an initial value per group. The second itself takes two arguments:
/// an 'accumulator', and an element. The second closure returns the value that the accumulator should have for the next iteration.
///
/// A special case of `fold`, in the spirit of SQL's GROUP BY and aggregation constructs.
/// The input is partitioned into groups by the first field, and for each group the values in the second field
/// are accumulated via the closures in the arguments.
///
/// ```hydroflow
/// recv_iter([("toy", 1), ("toy", 2), ("shoe", 11), ("shoe", 35), ("haberdashery", 7)])
///     -> groupby(|| 0, |old: &mut u32, val: u32| *old += val)
///     -> for_each(|(k, v)| println!("Total for group {} is {}", k, v));
///  ```
#[hydroflow_internalmacro::operator_docgen]
pub const GROUPBY: OperatorConstraints = OperatorConstraints {
    name: "groupby",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    ports_inn: None,
    ports_out: None,
    num_args: 2,
    input_delaytype_fn: &|_| Some(DelayType::Stratum),
    write_fn: &(|&WriteContextArgs { op_span, .. },
                 &WriteIteratorArgs {
                     ident,
                     inputs,
                     arguments,
                     is_pull,
                     ..
                 },
                 _| {
        assert!(is_pull);
        let input = &inputs[0];
        let initfn = &arguments[0];
        let aggfn = &arguments[1];
        let write_iterator = quote_spanned! {op_span=>
            let #ident = #input.fold(std::collections::HashMap::new(), |mut ht, nxt| {
                let e = ht.entry(nxt.0).or_insert_with(#initfn);
                #[allow(clippy::redundant_closure_call)]
                (#aggfn)(e, nxt.1);
                ht
            }).into_iter();
        };
        Ok(OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        })
    }),
};
