use std::hash::Hash;

use hydroflow_plus::*;
use location::NoTick;

#[expect(clippy::type_complexity, reason = "internal paxos code // TODO")]
pub fn collect_quorum<
    'a,
    L: Location<'a> + NoTick,
    Order,
    K: Clone + Eq + Hash,
    V: Clone,
    E: Clone,
>(
    tick: &Tick<L>,
    responses: Stream<(K, Result<V, E>), Tick<L>, Bounded, Order>,
    min: usize,
    max: usize,
) -> (
    Stream<(K, V), Tick<L>, Bounded, Order>,
    Stream<(K, E), Tick<L>, Bounded, Order>,
) {
    let (not_all_complete_cycle, not_all) = tick.cycle::<Stream<_, _, _, Order>>();

    let current_responses = not_all.union(responses.clone());

    let count_per_key = current_responses.clone().fold_keyed_commutative(
        q!(move || (0, 0)),
        q!(move |accum, value| {
            if value.is_ok() {
                accum.0 += 1;
            } else {
                accum.1 += 1;
            }
        }),
    );

    let not_reached_min_count =
        count_per_key
            .clone()
            .filter_map(q!(move |(key, (success, _error))| if success < min {
                Some(key)
            } else {
                None
            }));

    let reached_min_count =
        count_per_key
            .clone()
            .filter_map(q!(move |(key, (success, _error))| if success >= min {
                Some(key)
            } else {
                None
            }));

    let just_reached_quorum = if max == min {
        not_all_complete_cycle
            .complete_next_tick(current_responses.clone().anti_join(reached_min_count));

        current_responses.anti_join(not_reached_min_count)
    } else {
        let (min_but_not_max_complete_cycle, min_but_not_max) = tick.cycle();

        let received_from_all =
            count_per_key.filter_map(q!(
                move |(key, (success, error))| if (success + error) >= max {
                    Some(key)
                } else {
                    None
                }
            ));

        min_but_not_max_complete_cycle
            .complete_next_tick(reached_min_count.filter_not_in(received_from_all.clone()));

        not_all_complete_cycle.complete_next_tick(
            current_responses
                .clone()
                .anti_join(received_from_all.clone()),
        );

        current_responses
            .anti_join(not_reached_min_count)
            .anti_join(min_but_not_max)
    };

    (
        just_reached_quorum.filter_map(q!(move |(key, res)| match res {
            Ok(v) => Some((key, v)),
            Err(_) => None,
        })),
        responses.filter_map(q!(move |(key, res)| match res {
            Ok(_) => None,
            Err(e) => Some((key, e)),
        })),
    )
}
