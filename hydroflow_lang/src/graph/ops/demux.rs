use crate::graph::PortIndexValue;

use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_1,
};

use quote::{quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, ExprClosure};

// TODO(mingwei): Preprocess rustdoc links in mdbook or in the `operator_docgen` macro.

/// > Arguments: A Rust closure, the first argument is a received item and the
/// > second argument is a [`tl!` tuple list](https://hydro-project.github.io/hydroflow/doc/hydroflow/macro.tl.html)
/// > where each item name is an output port.
///
/// Takes the input stream and allows the user to determine what elemnt(s) to
/// deliver to any number of output streams.
///
/// > Note: Downstream operators may need explicit type annotations.
///
/// > Note: Import the [`Pusherator`](https://hydro-project.github.io/hydroflow/doc/pusherator/trait.Pusherator.html)
/// > trait to use the [`.give(...)` method](https://hydro-project.github.io/hydroflow/doc/pusherator/trait.Pusherator.html#tymethod.give).
///
/// ```hydroflow
/// my_demux = recv_iter(1..=100) -> demux(|v, tl!(fzbz, fizz, buzz, vals)|
///     match (v % 3, v % 5) {
///         (0, 0) => fzbz.give(v),
///         (0, _) => fizz.give(v),
///         (_, 0) => buzz.give(v),
///         (_, _) => vals.give(v),
///     }
/// );
/// my_demux[fzbz] -> for_each(|v| println!("{}: fizzbuzz", v));
/// my_demux[fizz] -> for_each(|v| println!("{}: fizz", v));
/// my_demux[buzz] -> for_each(|v| println!("{}: buzz", v));
/// my_demux[vals] -> for_each(|v| println!("{}", v));
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const DEMUX: OperatorConstraints = OperatorConstraints {
    name: "demux",
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: &(2..),
    soft_range_out: &(2..),
    ports_inn: None,
    ports_out: None,
    num_args: 1,
    input_delaytype_fn: &|_| None,
    write_fn: &(|&WriteContextArgs { root, op_span, .. },
                 &WriteIteratorArgs {
                     ident,
                     outputs,
                     output_ports,
                     arguments,
                     op_name,
                     is_pull,
                     ..
                 }| {
        assert!(!is_pull);
        let func = &arguments[0];
        let func: ExprClosure = parse_quote!(#func);
        assert_eq!(
            2,
            func.inputs.len(),
            "Closure provided as second argument to `{}` must have a second \
            input argument listing ports, `tl!(port_a, port_b, ...)`.",
            op_name,
        );
        let input_ports_str = func.inputs[1].to_token_stream().to_string();

        // A list of indices representing the permutation needed to reorder the
        // output port names to match how they appear in the `func` arguments.
        let mut sort_permute: Vec<_> = (0..outputs.len()).collect();
        sort_permute.sort_by_key(|&i| {
            if let PortIndexValue::Path(port_expr) = output_ports[i] {
                // Sort the index by the corresponding port's name location in
                // the argument string.
                let port_name = port_expr.to_token_stream().to_string();
                input_ports_str.find(&*port_name)
            } else {
                output_ports[i]
                    .span()
                    .unwrap()
                    .error("Port must be a valid identifier.");
                None
            }
        });

        let sorted_outputs = sort_permute.iter().map(|&i| &outputs[i]);

        let write_iterator = quote_spanned! {op_span=>
            let #ident = #root::pusherator::demux::Demux::new(#func, #root::tl!( #( #sorted_outputs ),* ));
        };
        OperatorWriteOutput {
            write_iterator,
            ..Default::default()
        }
    }),
};
