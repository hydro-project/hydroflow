# Hydroflow's Built-in Operators
In our previous examples we made use of some of Hydroflow's built-in operators.
Here we document each operators in more detail.

> *NOTE* Would be nice to format more like Rust docs, but I couldn't find that source as a template.
## filter
> 1 input stream, 1 output stream

> Arguments:
Filter outputs a subset of the items it receives at its input, according to a filter expression.

## map
> 1 input stream, 1 output stream

## flat_map
> 1 input stream, 1 output stream

## filter_map
> 1 input stream, 1 output stream

## merge
> _n_ input streams of the same type, 1 output stream of the same type

## join
> 2 input streams of type ..., 1 output stream of type ...

## tee
> 1 input stream, _n_ output streams

## recv_stream
> 0 input streams, 1 output stream

## recv_iter
> 0 input streams, 1 output stream

## for_each
> 1 input stream, 0 output streams
