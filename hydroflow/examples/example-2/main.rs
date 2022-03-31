use hydroflow::builder::prelude::*;

pub fn main() {
    let mut builder = HydroflowBuilder::new();
    builder.add_subgraph(
        "main",
        (0..10)
            .into_hydroflow()

            .map(|n| n * n)
            .filter(|&n| n > 10)

            .pull_to_push()

            .map(|n| (n-1..=n))
            .flatten()

            .for_each(|n| println!("Hello {}", n)),
    );

    let mut hydroflow = builder.build();
    hydroflow.tick();
}
