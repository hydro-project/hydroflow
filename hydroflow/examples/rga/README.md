# RGA 
This example illustrates a lattice-based tree for collaborative text editing, called a [Timestamped Insertion Tree](https://software.imdea.org/papers/2021-attiya-tcs.pdf) (Attiya et all 2016). It is not currently interactive; the `main` routine simply passes in the stream of "keystrokes" for the example. A keystroke is represented by a `(child, parent)` pair, where the child is a `Token` containing a (monotonically increasing) timestamp and a character, and the parent is the timestamp of the parent node in the tree. 

It then outputs a graph in the DOT format, which can be rendered with Graphviz -- this graph shows the tree structure, the total ordering of the tree nodes, and the string that comes from that ordering.

There are multiple implementations to choose from, via the `--impl` flag:
- A `minimal` implementation is nothing more than a set of (child, parent) edges. The Hydroflow code here does essentially nothing beyond collecting edges and outputting them.
- A 'datalog` implementation based on a [talk by Martin Kleppman](https://speakerdeck.com/ept/data-structures-as-queries-expressing-crdts-using-datalog). Kleppman's dataflow has been hand-compiled to Hydroflow, rule-by-rule.
- A 'datalog_agg` implementation that modifies Kleppman's code for Datalog-with-aggregation, and again is hand-compiled to Hydroflow
- A 'adjacency' implementation that builds an adjacency list in Hydroflow to reduce the number of aggregation passes. *This is the default if you do not specify the `--impl` flag.*

To run:
```
cargo run -p hydroflow --example rga -- -o <output file>
```
Optionally append `--impl <choice>` to choose an implementation among {`minimal`, `datalog`, `datalog_agg`, `adjacency`}
and append `--graph <type>` to also print a graph of the hydroflow code to stdout in one of the formats {`dot`, `mermaid`, `json`}.
