use hydroflow::compiled::pull::{JoinState, SymmetricHashJoin};
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, SeedableRng};
use std::hint::black_box;

fn main() {
    let mut rng = StdRng::from_entropy();

    let keys = Uniform::new(0, 1000usize);
    let values = Uniform::new(0, 1000000000usize);

    let lhs: Vec<_> = (0..10000000)
        .map(|_| (keys.sample(&mut rng), values.sample(&mut rng)))
        .collect();

    let rhs: Vec<_> = (0..10000000)
        .map(|_| (keys.sample(&mut rng), values.sample(&mut rng)))
        .collect();

    for _ in 0..1000 {
        let mut state = black_box(JoinState::default());
        let join = SymmetricHashJoin::new(
            black_box(lhs.iter().cloned()),
            black_box(rhs.iter().cloned()),
            &mut state,
        );

        for v in join {
            black_box(v);
        }
    }
}
