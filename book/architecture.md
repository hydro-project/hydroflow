# Architecture

Hydroflow graphs are divided into two layers: the outer _scheduled layer_ and
inner _compiled layer_.

The [Hydroflow Architecture Design Doc](https://www.mingweisamuel.com/hydroflow/design_docs/2021-10_architecture_design_doc.html)
contains a more detailed explanation of this section. Note that some aspects of
the design doc are not implemented (e.g. early yielding) or may become out of
date as time passes.

## Scheduled Layer

The scheduled layer is dynamic: it stores a list of operators (or "subgraphs")
as well as buffers ("handoffs") between them. The scheduled layer chooses how
to schedule the operators (naturally), and when each operator runs it pulls
from its input handoffs and pushes to its output handoffs. This setup is
extremely flexible: operators can have any number of input or output handoffs
so we can easily represent any graph topology. However this flexibility comes
at a performance cost due to the overhead of scheduling, buffering, and lack
of inter-operator compiler optimization.

The _compiled layer_ helps avoids the costs of the scheduled layer. We can
combine several operators into a _subgraph_ which are compiled and optimized as
a single scheduled subgraph. The scheduled layer then runs the entire subgraph
as if it was one operator with many inputs and outputs. Note the layering here:
the compiled layer does not replace the scheduled layer but instead exists
within it.

## Compiled Layer

Rust already has a built-in API similar to dataflow: [`Iterator`s](https://doc.rust-lang.org/std/iter/trait.Iterator.html).
When they work well, they work really well and tend to be as fast as for loops.
However, they are limited in the flow graphs they can represent. Each operator
in a `Iterator` takes ownership of, and _pulls_ from, the previous operator.
Taking two iterators as _inputs_ and merging them together (e.g. with
[`.chain(...)`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.chain)
or [`.zip(...)`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.zip))
is natural and performant as we can _pull_ from both. However if we want an
iterator to split into multiple _outputs_ then things become tricky. [`Itertools::tee()`](https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.tee)
does just this by cloning each incoming item into two output buffers, but this
requires allocation and could cause unbounded buffer growth if the outputs are
not read from evenly.

However, if instead iterators were _pull_-based, where each operator owns one
or more _output_ operators, then teeing is very easy, just clone each element
and push to (i.e. run) both outputs. So that's what we did, created push-based
iterators to allow fast teeing or splitting in the compiled layer.

Pull-based iterators can be connected to push-based iterators at a "pivot"
point. Together, this pull-to-push setup dictates the shape compiled subgraphs
can take. Informally, this is like the roots and leaves of a tree. Water flows
from the roots (the pull inputs), eventually all join together in the trunk
(the pull-to-push pivot), the split up into multiple outputs in the leaves.
We refer to this structure as an _in-out tree_.

||
| :---: |
| ![The Oaklandish Logo depicting the stylized roots, trunk, and branches of a tree.](img/oaklandish_logo.jpg) |
| The Oaklandish Logo. |

See [Subgraph In-Out Trees](./in-out_trees.md) for more, including how to
convert a graph into in-out trees.

## APIs

The two architectural layers can be interacted with through two corresponding
APIs. The _Surface API_ provides a natural `Iterator`-like chaining syntax for
building compiled subgraphs and using them as scheduled operators.

Alternatively, the _Core API_ requires interacting with handoffs directly and
doesn't provide any notion of chainable operators. You can use Rust `Iterator`s
or any other arbitrary Rust code to implement the operators.

We intend users to use the Surface API as it is much more friendly, but as
Hydroflow is in active development some operators might not be available in
the Surface API, in which case the Core API can be used instead. If you find
yourself in this sitation be sure to [submit an issue](https://github.com/hydro-project/hydroflow/issues/new)!

Surface API code lives in [`hydroflow::builder`](https://hydro-project.github.io/hydroflow/doc/hydroflow/builder/index.html)
while compiled push-based iterators live in [`hydroflow::compiled`](https://hydro-project.github.io/hydroflow/doc/hydroflow/compiled/index.html).
The Core API lives in [`hydroflow::scheduled`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/index.html),
mainly in methods on the [`Hydroflow` struct](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html).
## Scheduling

## Handoffs

## Networking
