---
sidebar_position: 7
---

# Graph Un-Reachability
> In this example we cover:
> * Extending a program with additional downstream logic.
> * Hydroflow's ([`difference`](../syntax/surface_ops_gen.md#difference)) operator
> * A first exposure to the concepts of _strata_ and _ticks_

Our next example builds on the previous by finding vertices that are _not_ reachable. To do this, we need to capture the set `all_vertices`, and use a [difference](../syntax/surface_ops_gen.md#difference) operator to form the difference between that set of vertices and `reachable_vertices`.

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

This is a simple augmentation of our previous example. Replace the contents of `src/main.rs` with the following:

```rust
use hydroflow::hydroflow_syntax;

pub fn main() {
    // An edge in the input data = a pair of `usize` vertex IDs.
    let (pairs_send, pairs_recv) = hydroflow::util::unbounded_channel::<(usize, usize)>();

    let mut flow = hydroflow_syntax! {
        origin = source_iter(vec![0]);
        stream_of_edges = source_stream(pairs_recv) -> tee();
        reached_vertices = union()->tee();
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

    println!(
        "{}",
        flow.meta_graph()
            .expect("No graph found, maybe failed to parse.")
            .to_mermaid()
    );

    pairs_send.send((5, 10)).unwrap();
    pairs_send.send((0, 3)).unwrap();
    pairs_send.send((3, 6)).unwrap();
    pairs_send.send((6, 5)).unwrap();
    pairs_send.send((11, 12)).unwrap();
    flow.run_available();
}
```
Notice that we are now sending in some new pairs to test this code. The output should be:
```console
#shell-command-next-line
cargo run
<build output>
<graph output>
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
[difference](../syntax/surface_ops_gen.md#difference) operator
to compute the difference between `all_vertices` and `reached_vertices`; note 
how we wire up the `pos` and `neg` inputs to the `difference` operator! 
Finally we print both `all_vertices` and `unreached_vertices`.

The auto-generated mermaid looks like so:
```mermaid
%%{init:{'theme':'base','themeVariables':{'clusterBkg':'#ddd','clusterBorder':'#888'}}}%%
flowchart TD
classDef pullClass fill:#02f,color:#999,stroke:#000,text-align:left,white-space:pre
classDef pushClass fill:#ff0,stroke:#000,text-align:left,white-space:pre
linkStyle default stroke:#aaa,stroke-width:4px,color:red,font-size:1.5em;
subgraph sg_1v1 ["sg_1v1 stratum 0"]
    1v1[\"(1v1) <code>source_iter(vec![0])</code>"/]:::pullClass
    8v1[\"(8v1) <code>map(|v| (v, ()))</code>"/]:::pullClass
    6v1[\"(6v1) <code>join()</code>"/]:::pullClass
    7v1[\"(7v1) <code>flat_map(|(src, ((), dst))| [src, dst])</code>"/]:::pullClass
    4v1[\"(4v1) <code>union()</code>"/]:::pullClass
    5v1[/"(5v1) <code>tee()</code>"\]:::pushClass
    15v1["(15v1) <code>handoff</code>"]:::otherClass
    15v1--->8v1
    1v1--0--->4v1
    8v1--0--->6v1
    6v1--->7v1
    7v1--1--->4v1
    4v1--->5v1
    5v1--0--->15v1
    subgraph sg_1v1_var_my_join ["var <tt>my_join</tt>"]
        6v1
        7v1
    end
    subgraph sg_1v1_var_origin ["var <tt>origin</tt>"]
        1v1
    end
    subgraph sg_1v1_var_reached_vertices ["var <tt>reached_vertices</tt>"]
        4v1
        5v1
    end
end
subgraph sg_2v1 ["sg_2v1 stratum 0"]
    2v1[\"(2v1) <code>source_stream(pairs_recv)</code>"/]:::pullClass
    3v1[/"(3v1) <code>tee()</code>"\]:::pushClass
    9v1[/"(9v1) <code>flat_map(|(src, dst)| [src, dst])</code>"\]:::pushClass
    10v1[/"(10v1) <code>tee()</code>"\]:::pushClass
    12v1[/"(12v1) <code>unique()</code>"\]:::pushClass
    13v1[/"(13v1) <code>for_each(|v| println!(&quot;Received vertex: {}&quot;, v))</code>"\]:::pushClass
    2v1--->3v1
    3v1--0--->9v1
    9v1--->10v1
    10v1--1--->12v1
    12v1--->13v1
    subgraph sg_2v1_var_all_vertices ["var <tt>all_vertices</tt>"]
        9v1
        10v1
    end
    subgraph sg_2v1_var_stream_of_edges ["var <tt>stream_of_edges</tt>"]
        2v1
        3v1
    end
end
subgraph sg_3v1 ["sg_3v1 stratum 1"]
    11v1[\"(11v1) <code>difference()</code>"/]:::pullClass
    14v1[/"(14v1) <code>for_each(|v| println!(&quot;unreached_vertices vertex: {}&quot;, v))</code>"\]:::pushClass
    11v1--->14v1
    subgraph sg_3v1_var_unreached_vertices ["var <tt>unreached_vertices</tt>"]
        11v1
    end
end
3v1--1--->16v1
5v1--1--->18v1
10v1--0--->17v1
16v1["(16v1) <code>handoff</code>"]:::otherClass
16v1--1--->6v1
17v1["(17v1) <code>handoff</code>"]:::otherClass
17v1--pos--->11v1
18v1["(18v1) <code>handoff</code>"]:::otherClass
18v1==neg===o11v1
```

## Strata and Ticks
Notice in the mermaid graph how Hydroflow separates the `difference` operator and its downstream dependencies into its own
_stratum_ (plural: _strata_). Note also the edge coming into the `neg` input to `difference` is bold and ends in a ball: this is because that input to `difference` is
"blocking", meaning that `difference` should not run until all of the input on that edge has been received.
The stratum boundary before `difference` ensures that the blocking property is respected.

Hydroflow runs each stratum
in order, one at a time, ensuring all values are computed
before moving on to the next stratum. Between strata we see a _handoff_, which logically buffers the 
output of the first stratum, and delineates the separation of execution between the 2 strata.

All the subgraphs labeled `stratum 0` are run first to completion, 
and then all the subgraphs labeled `stratum 1` are run. This captures the requirements of the `difference` operator: it has to wait for its full negative input before it can start producing output. Note
how the `difference` operator has two inputs (labeled `pos` and `neg`), and only the `neg` input shows up as blocking (with the bold edge ending in a ball).

Meanwhile, note stratum 0 has a recursive loop, and stratum 1 that computes `difference`, with the blocking input. This means that Hydroflow will first run the loop of stratum 0 repeatedly until all the transitive reached vertices are found, before moving on to compute the unreached vertices.

After all strata are run, Hydroflow returns to the stratum 0; this begins the next _tick_. This doesn't really matter for this example, but it is important for long-running Hydroflow services that accept input from the outside world. More on this topic in the chapter on [time](../concepts/life_and_times.md).


If you look carefully, you'll see two subgraphs labeled with `stratum 0`. The reason that stratum 0 was broken into subgraphs has nothing to do with
correctness, but rather the way that Hydroflow graphs are compiled and scheduled, as 
discussed in the chapter on [Architecture](../architecture/index.md).


