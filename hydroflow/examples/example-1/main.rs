use hydroflow::builder::prelude::*;

pub fn main() {
    let mut builder = HydroflowBuilder::new();
    builder.add_subgraph(
        "main",
        (0..10)
            .into_hydroflow()
            .pull_to_push()
            .for_each(|n| println!("Hello {}", n)),
    );

    let mut hydroflow = builder.build();
    hydroflow.tick();
}
