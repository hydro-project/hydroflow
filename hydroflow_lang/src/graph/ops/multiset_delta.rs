use quote::quote_spanned;

use super::{
    OperatorCategory, OperatorConstraints, OperatorWriteOutput,
    WriteContextArgs, RANGE_0, RANGE_1,
};

/// The multiset inverse of [`persist()`](#persist).
///
/// > 1 input stream of `T`, 1 output stream of `T`, where `T: Eq + Hash`
///
/// For set semantics, [`unique()`](#unique) can be thought of as a "delta" operator, the inverse
/// of [`persist()`](#persist). In `persist`, new items come in, and all items are repeatedly
/// released out. Conversely, `unique` take repeated items in, and only releases the new ones out.
///
/// This operator does a similar inversion but for multiset semantics, with some caveats. When it
/// receives duplicate items, instead of ignoring them, it "subtracts" them from the items received
/// in the previous tick: i.e. if we received `k` copies of an item in the previous tick, and we
/// receive `l > k` copies in the current tick, we output `l - k` copies of the item.
/// However unlike `unique`, this count is only maintained for the previous tick, not over all time.
///
/// In the example below, in the second tick two 'a's are removed because two 'a's were received in
/// the previous tick. The third 'a' is released though.
///
/// ```rustbook
/// let (input_send, input_recv) = hydroflow::util::unbounded_channel::<char>();
/// let mut flow = hydroflow::hydroflow_syntax! {
///     source_stream(input_recv)
///         -> multiset_delta()
///         -> for_each(|n| println!("{}", n));
/// };
///
/// input_send.send('a').unwrap();
/// input_send.send('b').unwrap();
/// input_send.send('a').unwrap();
/// flow.run_tick();
/// // 'a', 'b', 'a'
///
/// input_send.send('a').unwrap();
/// input_send.send('c').unwrap();
/// input_send.send('a').unwrap();
/// input_send.send('a').unwrap();
/// flow.run_tick();
/// // 'c', 'a'
/// // First two 'a's are removed due to previous tick.
///
/// input_send.send('b').unwrap();
/// input_send.send('c').unwrap();
/// input_send.send('a').unwrap();
/// input_send.send('a').unwrap();
/// input_send.send('a').unwrap();
/// input_send.send('a').unwrap();
/// flow.run_tick();
/// // 'b', 'a'
/// // 3 'a's and the 'c' are removed due to previous tick.
/// ```
pub const MULTISET_DELTA: OperatorConstraints = OperatorConstraints {
    name: "multiset_delta",
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
    input_delaytype_fn: |_| None,
    flow_prop_fn: None,
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

        let prev_data = wc.make_ident("prev_data");
        let curr_data = wc.make_ident("curr_data");

        let write_prologue = quote_spanned! {op_span=>
            let #prev_data = #hydroflow.add_state(::std::cell::RefCell::new(#root::rustc_hash::FxHashMap::default()));
            let #curr_data = #hydroflow.add_state(::std::cell::RefCell::new(#root::rustc_hash::FxHashMap::default()));
        };

        let tick_swap = quote_spanned! {op_span=>
            {
                if context.is_first_run_this_tick() {
                    let mut prev_map = #context.state_ref(#prev_data).borrow_mut();
                    let mut curr_map = #context.state_ref(#curr_data).borrow_mut();
                    ::std::mem::swap(::std::ops::DerefMut::deref_mut(&mut prev_map), ::std::ops::DerefMut::deref_mut(&mut curr_map));
                    curr_map.clear();
                }
            }
        };

        let filter_fn = quote_spanned! {op_span=>
            |item| {
                let mut prev_map = #context.state_ref(#prev_data).borrow_mut();
                let mut curr_map = #context.state_ref(#curr_data).borrow_mut();

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
                #tick_swap
                let #ident = #input.filter(#filter_fn);
            }
        } else {
            quote_spanned! {op_span=>
                #tick_swap
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
