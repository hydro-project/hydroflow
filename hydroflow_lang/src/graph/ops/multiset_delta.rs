use quote::quote_spanned;

use super::{
    FlowProperties, FlowPropertyVal, OperatorConstraints, OperatorWriteOutput, WriteContextArgs,
    RANGE_0, RANGE_1,
};

// TODO(mingwei): more doc
/// Multiset delta from the previous tick.
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<u32>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_stream(input_recv)
///         -> multiset_delta()
///         -> for_each(|n| println!("{}", n));
/// };
///
/// input_send.send(3).unwrap();
/// input_send.send(4).unwrap();
/// input_send.send(3).unwrap();
/// flow.run_tick();
/// // 3, 4, 3
///
/// input_send.send(3).unwrap();
/// input_send.send(5).unwrap();
/// input_send.send(3).unwrap();
/// input_send.send(3).unwrap();
/// flow.run_tick();
/// // 5, 3
/// // First two "3"s are removed due to previous tick.
/// ```
#[hydroflow_internalmacro::operator_docgen]
pub const MULTISET_DELTA: OperatorConstraints = OperatorConstraints {
    name: "multiset_delta",
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
        monotonic: FlowPropertyVal::Preserve,
        inconsistency_tainted: false,
    },
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   op_span,
                   context,
                   hydroflow,
                   ident,
                   inputs,
                   outputs,
                   is_pull,
                   ..
               },
               _| {
        let input = &inputs[0];
        let output = &outputs[0];

        let tick_data = wc.make_ident("tick_data");
        let prev_data = wc.make_ident("prev_data");
        let curr_data = wc.make_ident("curr_data");

        let write_prologue = quote_spanned! {op_span=>
            let #tick_data = #hydroflow.add_state(::std::cell::Cell::new(0_usize));
            let #prev_data = #hydroflow.add_state(::std::cell::RefCell::new(#root::rustc_hash::FxHashMap::default()));
            let #curr_data = #hydroflow.add_state(::std::cell::RefCell::new(#root::rustc_hash::FxHashMap::default()));
        };

        let filter_fn = quote_spanned! {op_span=>
            |item| {
                let tick = #context.state_ref(#tick_data);
                let mut prev_map = #context.state_ref(#prev_data).borrow_mut();
                let mut curr_map = #context.state_ref(#curr_data).borrow_mut();
                if tick.get() < #context.current_tick() {
                    tick.set(#context.current_tick());
                    ::std::mem::swap(&mut prev_map, &mut curr_map);
                    curr_map.clear();
                }

                *curr_map.entry(#[allow(clippy::clone_on_copy)] item.clone()).or_insert(0_usize) += 1;
                if let Some(old_count) = prev_map.get_mut(item) {
                    #[allow(clippy::absurd_extreme_comparisons)] // Usize cannot be less than zero.
                    if *old_count <= 0 {
                        true
                    } else {
                        *old_count -= 1;
                        false
                    }
                } else {
                    true
                }
            }
        };
        let write_iterator = if is_pull {
            quote_spanned! {op_span=>
                let #ident = #input.filter(#filter_fn);
            }
        } else {
            quote_spanned! {op_span=>
                let #ident = #root::pusherator::filter::Filter::new(#filter_fn, #output);
            }
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
