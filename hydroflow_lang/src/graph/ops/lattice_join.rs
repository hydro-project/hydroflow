use quote::{quote_spanned, ToTokens};
use syn::parse_quote;

use super::{
    FlowProperties, FlowPropertyVal, OperatorCategory, OperatorConstraints, OperatorWriteOutput,
    Persistence, WriteContextArgs, RANGE_1,
};
use crate::diagnostic::{Diagnostic, Level};
use crate::graph::{OpInstGenerics, OperatorInstance};

/// > 2 input streams of type <(K, V1)> and <(K, V2)>, 1 output stream of type <(K, (V1, V2))>
///
/// Performs a group-by with lattice-merge aggregate function on LHS and RHS inputs and then forms the
/// equijoin of the tuples in the input streams by their first (key) attribute. Note that the result nests the 2nd input field (values) into a tuple in the 2nd output field.
///
/// You must specify the LHS and RHS lattice types, they cannot be inferred.
///
/// ```hydroflow
/// // should print `(key, (2, 1))`
/// my_join = lattice_join::<hydroflow::lattices::Max<usize>, hydroflow::lattices::Max<usize>>();
/// source_iter(vec![("key", hydroflow::lattices::Max::new(0)), ("key", hydroflow::lattices::Max::new(2))]) -> [0]my_join;
/// source_iter(vec![("key", hydroflow::lattices::Max::new(1))]) -> [1]my_join;
/// my_join -> for_each(|(k, (v1, v2))| println!("({}, ({:?}, {:?}))", k, v1, v2));
/// ```
///
/// `lattice_join` can also be provided with one or two generic lifetime persistence arguments, either
/// `'tick` or `'static`, to specify how join data persists. With `'tick`, pairs will only be
/// joined with corresponding pairs within the same tick. With `'static`, pairs will be remembered
/// across ticks and will be joined with pairs arriving in later ticks. When not explicitly
/// specified persistence defaults to `static.
///
/// When two persistence arguments are supplied the first maps to port `0` and the second maps to
/// port `1`.
/// When a single persistence argument is supplied, it is applied to both input ports.
/// When no persistence arguments are applied it defaults to `'static` for both.
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
/// ```rustbook
/// use hydroflow::lattices::Min;
/// use hydroflow::lattices::Max;
///
/// let mut df = hydroflow::hydroflow_syntax! {
///     my_join = lattice_join::<'tick, Min<usize>, Max<usize>>();
///     source_iter([(7, Min::new(1)), (7, Min::new(2))]) -> [0]my_join;
///     source_iter([(7, Max::new(1)), (7, Max::new(2))]) -> [1]my_join;
///     my_join -> assert([(7, (Min::new(1), Max::new(2)))]);
/// };
/// df.run_available();
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
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
                   root,
                   context,
                   op_span,
                   ident,
                   inputs,
                   op_inst:
                       OperatorInstance {
                           generics:
                               OpInstGenerics {
                                   persistence_args,
                                   type_args,
                                   ..
                               },
                           ..
                       },
                   ..
               },
               diagnostics| {
        let lhs_type = type_args
            .get(0)
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote_spanned!(op_span=> _));

        let rhs_type = type_args
            .get(1)
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote_spanned!(op_span=> _));

        let mut make_joindata = |persistence, side| {
            let joindata_ident = wc.make_ident(format!("joindata_{}", side));
            let borrow_ident = wc.make_ident(format!("joindata_{}_borrow", side));
            let (init, borrow) = match persistence {
                Persistence::Tick => (
                    quote_spanned! {op_span=>
                        #root::util::monotonic_map::MonotonicMap::new_init(
                            #root::compiled::pull::HalfJoinStateLattice::default()
                        )
                    },
                    quote_spanned! {op_span=>
                        &mut *#borrow_ident.get_mut_clear(#context.current_tick())
                    },
                ),
                Persistence::Static => (
                    quote_spanned! {op_span=>
                        #root::compiled::pull::HalfJoinStateLattice::default()
                    },
                    quote_spanned! {op_span=>
                        &mut #borrow_ident
                    },
                ),
                Persistence::Mutable => {
                    diagnostics.push(Diagnostic::spanned(
                        op_span,
                        Level::Error,
                        "An implementation of 'mutable does not exist",
                    ));
                    return Err(());
                }
            };
            Ok((joindata_ident, borrow_ident, init, borrow))
        };

        let persistences = match persistence_args[..] {
            [] => [Persistence::Static, Persistence::Static],
            [a] => [a, a],
            [a, b] => [a, b],
            _ => unreachable!(),
        };

        let (lhs_joindata_ident, lhs_borrow_ident, lhs_init, lhs_borrow) =
            make_joindata(persistences[0], "lhs")?;
        let (rhs_joindata_ident, rhs_borrow_ident, rhs_init, rhs_borrow) =
            make_joindata(persistences[1], "rhs")?;

        let join_keys_ident = wc.make_ident("joinkeys");
        let join_keys_borrow_ident = wc.make_ident("joinkeys_borrow");

        let write_prologue = quote_spanned! {op_span=>
            let #lhs_joindata_ident = df.add_state(::std::cell::RefCell::new(
                #lhs_init
            ));
            let #rhs_joindata_ident = df.add_state(::std::cell::RefCell::new(
                #rhs_init
            ));
            let #join_keys_ident = df.add_state(::std::cell::RefCell::new(
                #root::rustc_hash::FxHashSet::default()
            ));
        };

        let lhs = &inputs[0];
        let rhs = &inputs[1];
        let write_iterator = quote_spanned! {op_span=>
            let mut #lhs_borrow_ident = #context.state_ref(#lhs_joindata_ident).borrow_mut();
            let mut #rhs_borrow_ident = #context.state_ref(#rhs_joindata_ident).borrow_mut();
            let mut #join_keys_borrow_ident = #context.state_ref(#join_keys_ident).borrow_mut();

            let #ident = {
                /// Limit error propagation by bounding locally, erasing output iterator type.
                #[inline(always)]
                fn check_inputs<'a, Key, I1, Lhs, LhsDelta, I2, Rhs, RhsDelta>(
                    lhs: I1,
                    rhs: I2,
                    updated_keys: &'a mut #root::rustc_hash::FxHashSet<Key>,
                    lhs_state: &'a mut #root::compiled::pull::HalfJoinStateLattice<Key, Lhs>,
                    rhs_state: &'a mut #root::compiled::pull::HalfJoinStateLattice<Key, Rhs>,
                ) -> impl 'a + Iterator<Item = (Key, (Lhs, Rhs))>
                where
                    Key: Eq + std::hash::Hash + Clone,
                    Lhs: #root::lattices::Merge<LhsDelta> + Clone + #root::lattices::LatticeFrom<LhsDelta>,
                    Rhs: #root::lattices::Merge<RhsDelta> + Clone + #root::lattices::LatticeFrom<RhsDelta>,
                    I1: Iterator<Item = (Key, LhsDelta)>,
                    I2: Iterator<Item = (Key, RhsDelta)>,
                {
                    #root::compiled::pull::SymmetricHashJoinLattice::new_from_mut(lhs, rhs, updated_keys, lhs_state, rhs_state)
                }
                check_inputs::<_, _, #lhs_type, _, _, #rhs_type, _>(#lhs, #rhs, &mut *#join_keys_borrow_ident, #lhs_borrow, #rhs_borrow)
            };
        };

        Ok(OperatorWriteOutput {
            write_prologue,
            write_iterator,
            ..Default::default()
        })
    },
};
