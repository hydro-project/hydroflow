use super::{
    OperatorConstraints, OperatorWriteOutput, WriteContextArgs, WriteIteratorArgs, RANGE_0, RANGE_1,
};

use crate::diagnostic::{Diagnostic, Level};
use quote::quote_spanned;
use syn::Expr;

#[hydroflow_internalmacro::operator_docgen]
pub const BATCH_ITER_FN: OperatorConstraints = OperatorConstraints {
    name: "batch_iter_fn",
    hard_range_inn: RANGE_0,
    soft_range_inn: RANGE_0,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 2,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
    is_external_input: false,
    ports_inn: None,
    ports_out: None,
    input_delaytype_fn: &|_| None,
    write_fn: &(|wc @ &WriteContextArgs {
                     context, op_span, ..
                 },
                 &WriteIteratorArgs {
                     ident, arguments, ..
                 },
                 diagnostics| {
        let func = &arguments[1];
        let Expr::Closure(func) = func else {
                        diagnostics.push(Diagnostic::spanned(
                            op_span,
                            Level::Error,
                            "Second argument must be a 0 argument closure expression"),
                        );
                        return Err(());
                    };

        if 0 != func.inputs.len() {
            diagnostics.push(Diagnostic::spanned(
                op_span,
                Level::Error,
                &*format!("badbad",),
            ));
            return Err(());
        }

        let gen_ident = wc.make_ident("gen_fun");

        let write_prologue = quote_spanned! {op_span=>
            let mut #gen_ident = #func;
        };

        let arg1 = &arguments[0];

        let write_iterator = quote_spanned! {op_span=>
            let #ident = {
                (0..#arg1).map(|_| #gen_ident())
            };
        };
        let write_iterator_after = quote_spanned! {op_span=>
            #context.schedule_subgraph(#context.current_subgraph());
        };
        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
            ..Default::default()
        })
    }),
};
