# Graph Reachability

So far all the operators we've used have one input and one output and therefore
create a linear graph. Let's now take a look at a Hydroflow program containing
a subgraph which has multiple inputs and outputs.
To motivate this, we'll tackle the simple problem of graph reachability. Given
a graph in the form of a streaming list of edges, which vertices can be reached
from the origin vertex?

Let's dive straight into the full code:

```rust
use hydroflow::builder::prelude::*;

pub fn main() {
    let mut builder = HydroflowBuilder::default();

    let (send_edges, recv_edges) =
        builder.add_channel_input::<_, _, VecHandoff<(usize, usize)>>("edge input");
    let (send_loop, recv_loop) = builder.make_edge::<_, VecHandoff<usize>, _>("loop");

    builder.add_subgraph(
        "main",
        std::iter::once(0)
            .into_hydroflow()
            .chain(recv_loop.flatten())
            .map(|v| (v, ()))
            .join(recv_edges.flatten())
            .pull_to_push()
            .map(|(_old_v, (), new_v)| new_v)
            .tee(
                builder.start_tee().for_each(|v| println!("Reached: {}", v)),
                builder.start_tee().map(Some).push_to(send_loop),
            ),
    );

    let mut hf = builder.build();
    println!("{}", hf.generate_mermaid());

    println!("A");
    send_edges.give(Some((5, 10)));
    send_edges.give(Some((0, 3)));
    send_edges.give(Some((3, 6)));
    send_edges.flush();
    hf.tick();

    println!("B");
    send_edges.give(Some((6, 5)));
    send_edges.flush();
    hf.tick();
}
```
```txt
A
Reached: 3
Reached: 6
B
Reached: 5
Reached: 10
```
```mermaid
graph TD
  subgraph stratum0
    subgraph main1
      1.2[Chain] --> 1.1[Map]
      1.11[Map] --> Handoff_1[\loop/]
      1.8[Map] --> 1.9[Tee]
      1.1[Map] --> 1.0[Join]
      1.0[Join] --> 1.13[/PullToPush\]
      1.3[Iter] --> 1.2[Chain]
      1.4[Flatten] --> 1.2[Chain]
      1.13[/PullToPush\] --> 1.8[Map]
      Handoff_0[\edge input handoff/] --> 1.6[Flatten]
      1.6[Flatten] --> 1.0[Join]
      1.9[Tee] --> 1.10[ForEach]
      Handoff_1[\loop/] --> 1.4[Flatten]
      1.9[Tee] --> 1.11[Map]
    end
  end
```

TODO
