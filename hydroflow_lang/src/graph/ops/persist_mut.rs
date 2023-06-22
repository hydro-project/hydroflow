use quote::quote_spanned;

use super::{
    DelayType, FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints,
    OperatorWriteOutput, WriteContextArgs, RANGE_0, RANGE_1,
};

/// `persist_mut()` is similar to `persist()` except that it also enables deletions.
/// `persist_mut()` expects an input of type `Persistence<T>`, and it is this enumeration that enables the user to communicate deletion.
/// Deletions/persists hapepn in the order they are received in the stream. For example, [Persist(1), Delete(1), Persist(1)] will result ina a single '1' value being stored.
///
/// ```hydroflow
/// source_iter([hydroflow::util::Persistence::Persist(1), hydroflow::util::Persistence::Persist(2), hydroflow::util::Persistence::Delete(1)])
///     -> persist_mut()
///     -> assert([2]);
/// ```
pub const PERSIST_MUT: OperatorConstraints = OperatorConstraints {
    name: "persist_mut",
    categories: &[OperatorCategory::Persistence],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: RANGE_1,
    soft_range_out: RANGE_1,
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_0,
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
                   context,
                   hydroflow,
                   op_span,
                   ident,
                   inputs,
                   is_pull,
                   ..
               },
               _| {
        assert!(is_pull);

        let persistdata_ident = wc.make_ident("persistdata");
        let vec_ident = wc.make_ident("persistvec");
        let tick_ident = wc.make_ident("persisttick");
        let write_prologue = quote_spanned! {op_span=>
            let #persistdata_ident = #hydroflow.add_state(::std::cell::RefCell::new((
                0_usize, // tick
                #root::util::sparse_vec::SparseVec::default(),
            )));
        };

        let write_iterator = {
            let input = &inputs[0];
            quote_spanned! {op_span=>
                let mut #vec_ident = #context.state_ref(#persistdata_ident).borrow_mut();
                let (ref mut #tick_ident, ref mut #vec_ident) = &mut *#vec_ident;
                let #ident = {
                    #[inline(always)]
                    fn check_iter<T: ::std::hash::Hash + ::std::cmp::Eq>(iter: impl Iterator<Item = #root::util::Persistence::<T>>) -> impl Iterator<Item = #root::util::Persistence::<T>> {
                        iter
                    }

                    if *#tick_ident <= #context.current_tick() {
                        *#tick_ident = 1 + #context.current_tick();

                        for item in check_iter(#input) {
                            match item {
                                #root::util::Persistence::Persist(v) => #vec_ident.push(v),
                                #root::util::Persistence::Delete(v) => #vec_ident.delete(&v),
                            }
                        }

                        Some(#vec_ident.iter().cloned()).into_iter().flatten()
                    } else {
                        None.into_iter().flatten()
                    }
                };
            }
        };

        let write_iterator_after = quote_spanned! {op_span=>
            #context.schedule_subgraph(#context.current_subgraph(), false);
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            write_iterator_after,
        })
    },
};
