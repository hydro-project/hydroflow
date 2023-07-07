---
sidebar_position: 2
---

# Flow Syntax
Flows consist of named _operators_ that are connected via flow _edges_ denoted by `->`. The example below
uses the [`source_iter`](./surface_ops_gen.md#source_iter) operator to generate two strings from a Rust `vec`, the
[`map`](./surface_ops_gen.md#map) operator to apply some Rust code to uppercase each string, and the [`for_each`](./surface_ops_gen.md#for_each)
operator to print each string to stdout.
```rust,ignore
source_iter(vec!["Hello", "world"])
    -> map(|x| x.to_uppercase()) -> for_each(|x| println!("{}", x));
```

Flows can be assigned to variable names for convenience. E.g, the above can be rewritten as follows:
```rust,ignore
source_iter(vec!["Hello", "world"]) -> upper_print;
upper_print = map(|x| x.to_uppercase()) -> for_each(|x| println!("{}", x));
```
Note that the order of the statements (lines) doesn't matter. In this example, `upper_print` is
referenced before it is assigned, and that is completely OK and better matches the flow of
data, making the program more understandable.

## Operators with Multiple Ports
Some operators have more than one input _port_ that can be referenced by `->`. For example [`union`](./surface_ops_gen.md#union)
unions the contents of many flows, so it can have an abitrary number of input ports. Some operators have multiple outputs, notably [`tee`](./surface_ops_gen.md#tee), and [`demux`](./surface_ops_gen.md#demux)
which have an arbitrary number of outputs.

In the syntax, we optionally distinguish input ports via an _indexing prefix_ string
in square brackets before the name (e.g. `[0]my_union` and `[1]my_union`). Binary operators—those with two distinct input ports—require indexing prefixes to distinguish them.
Operators with arbitrary numbers of inputs ([`union`](./surface_ops_gen.md#union)) and outputs 
([`demux`](./surface_ops_gen.md#demux), [`tee`](./surface_ops_gen.md#tee)) 
allow for arbitrary strings, which can make code and dataflow graphs more readable and understandable
(e.g. `my_tee[print]` and `my_tee[continue]`).

Here is an example that tees one flow into two, handles each separately, and then unions them to print out the contents in both lowercase and uppercase:
```rust,ignore
my_tee = source_iter(vec!["Hello", "world"]) -> tee();
my_tee -> map(|x| x.to_uppercase()) -> [low_road]my_union;
my_tee -> map(|x| x.to_lowercase()) -> [high_road]my_union;
my_union = union() -> for_each(|x| println!("{}", x));
```
Here is a visualization of the flow that was generated. Note that the outbound labels to `my_tee` 
were auto-generated, but the inbound labels to `my_union` were specified by the code above:
```mermaid
%%{init:{'theme':'base','themeVariables':{'clusterBkg':'#ddd','clusterBorder':'#888'}}}%%
flowchart TD
classDef pullClass fill:#02f,color:#fff,stroke:#000
classDef pushClass fill:#ff0,stroke:#000
linkStyle default stroke:#aaa,stroke-width:4px,color:red,font-size:1.5em;
subgraph sg_1v1 ["sg_1v1 stratum 0"]
    1v1[\"(1v1) <tt>source_iter(vec! [&quot;Hello&quot;, &quot;world&quot;])</tt>"/]:::pullClass
    2v1[/"(2v1) <tt>tee()</tt>"\]:::pushClass
    1v1--->2v1
    subgraph sg_1v1_var_my_tee ["var <tt>my_tee</tt>"]
        1v1
        2v1
    end
end
subgraph sg_2v1 ["sg_2v1 stratum 0"]
    3v1[\"(3v1) <tt>map(| x : &amp; str | x.to_uppercase())</tt>"/]:::pullClass
    4v1[\"(4v1) <tt>map(| x : &amp; str | x.to_lowercase())</tt>"/]:::pullClass
    5v1[\"(5v1) <tt>union()</tt>"/]:::pullClass
    6v1[/"(6v1) <tt>for_each(| x | println! (&quot;{}&quot;, x))</tt>"\]:::pushClass
    3v1--low road--->5v1
    4v1--high road--->5v1
    5v1--->6v1
    subgraph sg_2v1_var_my_union ["var <tt>my_union</tt>"]
        5v1
        6v1
    end
end
2v1--0--->7v1
2v1--1--->8v1
7v1["(7v1) <tt>handoff</tt>"]:::otherClass
7v1--->3v1
8v1["(8v1) <tt>handoff</tt>"]:::otherClass
8v1--->4v1
```
Hydroflow compiled this flow into two subgraphs called _compiled components_, connected by _handoffs_. You can ignore
these details unless you are interested in low-level performance tuning; they are explained in the discussion
of [in-out trees](../architecture/in-out_trees.md).
## The `context` object

Closures inside surface syntax operators have access to a special `context` object which provides
access to scheduling, timing, and state APIs. The object is accessible as a shared reference
(`&Context`) via the special name `context`.
[Here is the full API documentation for `Context`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/context/struct.Context.html).

```rust,ignore
source_iter([()])
    -> for_each(|()| println!("Current tick: {}, stratum: {}", context.current_tick(), context.current_stratum()));
// Current tick: 0, stratum: 0
```
