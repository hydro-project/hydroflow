# TODO

- **Document the methods on the `hydroflow` trait** -- especially the run methods.
    -  The [`run_epoch()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_epoch), [`run_stratum()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_stratum), [`run()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run), and [`run_async()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_async) methods provide other ways to control the graph execution.

- **Strata and Epochs** -- explain the concept of strata and epochs, and how they relate to the `run_epoch()` and `run_stratum()` methods.
Before we proceed, note in the mermaid graph how Hydroflow separates the `unique` operator and its downstream dependencies into their own
_stratum_ (plural: _strata_). The stratum boundary before `unique` ensures that all the values arrive before it executes, ensuring that all duplicates are eliminated. 

Hydroflow runs each stratum
in order, one at a time, ensuring all values are computed
before moving on to the next stratum. Between strata we see a _handoff_, which logically buffers the 
output of the first stratum, and delineates the separation of execution between the 2 strata.

After all strata are run, Hydroflow returns to the first stratum; this begins the next _epoch_. This doesn't really matter for this example, but it is important for long-running Hydroflow services that accept input from the outside world.

- **Make sure `src/examples/echoserver` is the same as the template project** -- or better, find a way to do that via github actions or a github submodule