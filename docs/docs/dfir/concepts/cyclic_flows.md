---
sidebar_position: 2
---

import CodeBlock from '@theme/CodeBlock';
import exampleCode from '!!raw-loader!../../../../dfir_rs/examples/example_5_reachability.rs';
import exampleCode2 from '!!raw-loader!../../../../dfir_rs/examples/example_naturals.rs';
import { getLines, extractOutput, extractMermaid } from '../../../src/util';

# Dataflow Cycles and Fixpoints
Many dataflow libraries only support acyclic flow graphs (DAGs); DFIR goes further and supports cycles. DFIR's semantics for cyclic flows are based on the formal foundations of recursive queries in the [Datalog](https://en.wikipedia.org/wiki/Datalog) language, which also influenced the design of recursive query features in SQL.

The basic pattern for cycles in DFIR looks something like this:
```
        base = source_<XXX>() -> ... -> [base]cycle;
        cycle = union() 
                -> ... 
                -> [next]cycle;
```
That is, we can trace a cycle of operators in the graph, where one operator is a `union()` that accepts two inputs, one of which is the "back edge" that closes the cycle. 

For a concrete example, we can revisit the code in the [Graph Reachability](../quickstart/example_5_reachability.mdx) quickstart program:

<CodeBlock language="rust">{getLines(exampleCode, 7, 22)}</CodeBlock>

The cycle in that program matches our rough pattern as follows:
```
        origin = source_iter(vec![0]) -> [base]reached_vertices;
        reached_vertices = union() -> map(...) 
                           -> [0]my_join_tee 
                           -> ... 
                           -> [next]reached_vertices;
```

How should we think about a cycle like this? Intuitively, we can think of the cycle beginning to compute on the data from `base` that comes in via `[0]cycle`. In the Graph Reachability example, this base data corresponds to the origin vertex, `0`. By joining [0] with the `stream_of_edges`, we generate neighbors (1 hop away) and pass them back into the cycle. When one of these is joined again with `stream_of_edges` we get a neighbor of a neighbor (2 hops away). When one of *these* is joined with `stream_of_edges` we get a vertex 3 hops away, and so on. 

If you prefer to think about this program as logic, it represents [Mathematical Induction](https://en.wikipedia.org/wiki/Mathematical_induction) via dataflow: the data from `base` going into `[0]cycle` (i.e. the origin vertex, 0 hops away) is like a "base case", and the data going into `[1]cycle` represents the "induction step" (a vertex *k+1* hops away). (A graph with multiple cycles represents multiple induction, which is a relatively uncommon design pattern in both mathematics and DFIR!)

When does this process end? As with most DFIR questions, the answer is not in terms of control flow, but rather in terms of dataflow: *the cycle terminates when it produces no new data*, a notion called a [fixpoint](https://en.wikipedia.org/wiki/Fixed_point_(mathematics)). Our graph reachability example, it terminates when there are no new vertices to visit. Note that the `[join()](../syntax/surface_ops_gen#join)` operator is defined over the *sets* of inputs on each side, and sets
by definition do not contain duplicate values. This prevents the Reachability dataflow from regenerating the same value multiple times.

Like many looping constructs, it is possible to write a cyclic DFIR program that never ``terminates``, in the sense that it produces an unbounded stream of data. If we use `[join_multiset()](../syntax/surface_ops_gen#join_multiset)` instead of `[join()](../syntax/surface_ops_gen#join)` in our Reachability dataflow, the call to `flow.run_available()` never terminates, because each time the same vertex is visited, new data is generated!

A simpler example of a non-terminating cycle is the following, which specifies the natural numbers:

<CodeBlock language="rust" showLineNumbers>{exampleCode2}</CodeBlock>

Like any sufficiently powerful language, DFIR cannot guarantee that your programs terminate. If you're debugging a non-terminating DFIR program, it's a good idea to identify the dataflow cycles and insert an
`[inspect()](../syntax/surface_ops_gen#inspect)` operator along an edge of the cycle to see if it's generating unbounded amounts of duplicated data. You can use the `[unique()](../syntax/surface_ops_gen#unique)` operator to ameliorate the problem.