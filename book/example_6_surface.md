# Graph Un-Reachability
> In this example we cover:
> * Extending a program with additional downstream logic.
> * Hydroflow's ([`difference`](./surface_ops.gen.md#merge)) operator
> * Further examples of automatic stratification.

Our next example builds on the previous by finding vertices that are _not_ reachable. To do this, we need to capture the set `all_vertices`, and use a [difference](./surface_ops.gen.md#difference) operator to form the difference between that set of vertices and `reachable_vertices`.

Essentially we want a flow like this:
```mermaid
graph TD
  subgraph sources
    01[Stream of Edges]
  end
  subgraph reachable from origin
    00[Origin Vertex]
    10[Reached Vertices]
    20("V ⨝ E")

    00 --> 10
    10 --> 20
    20 --> 10

    01 --> 20
  end
  subgraph unreachable
    15[All Vertices]
    30(All - Reached)
    01 ---> 15
    15 --> 30
    10 --> 30
    30 --> 40
   end
40[Output]
```

This is a simple augmentation of our previous example. Here's the code:

```rust
# use hydroflow::hydroflow_syntax;
pub fn main() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut df = hydroflow_syntax! {
        origin = recv_iter(vec![0]);
        stream_of_edges = recv_stream(pairs_recv) -> tee();
        reached_vertices = merge()->tee();
        origin -> [0]reached_vertices;

        // the join for reachable vertices
        my_join = join() -> flat_map(|(src, ((), dst))| [src, dst]);
        reached_vertices[0] -> map(|v| (v, ())) -> [0]my_join;
        stream_of_edges[1] -> [1]my_join;

        // the loop
        my_join -> [1]reached_vertices;

        // the difference all_vertices - reached_vertices
        all_vertices = stream_of_edges[0]
          -> flat_map(|(src, dst)| [src, dst]) -> tee();
        unreached_vertices = difference();
        all_vertices[0] -> [pos]unreached_vertices;
        reached_vertices[1] -> [neg]unreached_vertices;

        // the output
        all_vertices[1] -> unique() -> for_each(|v| println!("Received vertex: {}", v));
        unreached_vertices -> for_each(|v| println!("unreached_vertices vertex: {}", v));
    };

    if let Some(graph) = opts.graph {
        let serde_graph = df
            .serde_graph()
            .expect("No graph found, maybe failed to parse.");
        match graph {
            GraphType::Mermaid => {
                println!("{}", serde_graph.to_mermaid());
            }
            GraphType::Dot => {
                println!("{}", serde_graph.to_dot())
            }
            GraphType::Json => {
                unimplemented!();
                // println!("{}", serde_graph.to_json())
            }
        }
    }

    pairs_send.send((5, 10)).unwrap();
    pairs_send.send((0, 3)).unwrap();
    pairs_send.send((3, 6)).unwrap();
    pairs_send.send((6, 5)).unwrap();
    pairs_send.send((11, 12)).unwrap();
    df.run_available();
}
```
Notice that we are now sending in some new pairs to test this code. The output should be:
```txt
Received vertex: 12
Received vertex: 6
Received vertex: 11
Received vertex: 0
Received vertex: 5
Received vertex: 10
Received vertex: 3
unreached_vertices vertex: 12
unreached_vertices vertex: 11
```

Let's review the changes, all of which come at the end of the program. First, 
we remove code to print `reached_vertices`. Then we define `all_vertices` to be
the vertices that appear in any edge (using familiar `flat_map` code from the previous 
examples.) In the last few lines, we wire up a 
[difference](./surface_ops.gen.md#difference) operator
to compute the difference between `all_vertices` and `reached_vertices`; note 
how we wire up the `pos` and `neg` inputs to the `difference` operator! 
Finally we print both `all_vertices` and `unreached_vertices`.

The auto-generated mermaid looks like so:
```mermaid
flowchart TB
    subgraph "sg_1v1 stratum 0"
        1v1["1v1 <tt>op_1v1: recv_iter(vec! [0])</tt>"]
        8v1["8v1 <tt>op_8v1: map(| v | (v, ()))</tt>"]
        6v1["6v1 <tt>op_6v1: join()</tt>"]
        7v1["7v1 <tt>op_7v1: flat_map(| (src, ((), dst)) | [src, dst])</tt>"]
        4v1["4v1 <tt>op_4v1: merge()</tt>"]
        5v1["5v1 <tt>op_5v1: tee()</tt>"]
    end
    subgraph "sg_2v1 stratum 0"
        2v1["2v1 <tt>op_2v1: recv_stream(pairs_recv)</tt>"]
        3v1["3v1 <tt>op_3v1: tee()</tt>"]
        9v1["9v1 <tt>op_9v1: flat_map(| (src, dst) | [src, dst])</tt>"]
        10v1["10v1 <tt>op_10v1: tee()</tt>"]
    end
    subgraph "sg_3v1 stratum 1"
        12v1["12v1 <tt>op_12v1: unique()</tt>"]
        13v1["13v1 <tt>op_13v1: for_each(| v | println! (&quot;Received vertex: {}&quot;, v))</tt>"]
    end
    subgraph "sg_4v1 stratum 1"
        11v1["11v1 <tt>op_11v1: difference()</tt>"]
        14v1["14v1 <tt>op_14v1: for_each(| v | println! (&quot;unreached_vertices vertex: {}&quot;, v))</tt>"]
    end

    15v1{"handoff"}
    16v1{"handoff"}
    17v1{"handoff"}
    18v1{"handoff"}
    19v1{"handoff"}

    1v1-->4v1
    2v1-->3v1
    3v1-->16v1
    3v1-->9v1
    4v1-->5v1
    5v1-->15v1
    5v1-->18v1
    6v1-->7v1
    7v1-->4v1
    8v1-->6v1
    9v1-->10v1
    10v1-->17v1
    10v1-->19v1
    11v1-->14v1
    12v1-->13v1
    15v1-->8v1
    16v1-->6v1
    17v1-->11v1
    18v1-->11v1
    19v1-->12v1
```
If you look carefully, you'll see two subgraphs labeled with `stratum 0`, and two with
`stratum 1`. All the subgraphs labeled `stratum 0` are run first to completion, 
and then all the subgraphs labeled `stratum 1` are run. This captures the requirements of the `unique` and `difference` operators used in the lower subgraphs: each has to wait for its full inputs before it can start producing output.