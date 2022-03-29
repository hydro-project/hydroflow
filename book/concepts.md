# Concepts

## Dataflow and Networking

Conventionally, dataflow systems provide a way to analyze and process data by
chaining functions (operators) together into pipelines. But with a change of
perspective, many computations can be represented as dataflow. Designing around
the flow of data naturally separates computation into easily parallelizable and
distributable pipelines.

Most dataflow systems provide opinionated mechanisms for running flows across
multiple machines. Hydroflow however aims to represent many more types of
computations besides just data processing. This includes networking protocols
like two-phase commit and Paxos. To this end, Hydroflow is unopinionated about
network communication. Additionally, Hydroflow provides the abstraction of only
a single node (single thread) of a system. To build up a distributed system,
the user must design each node to communicate with others as needed.

As development continues we will provide more utilities for common networking
use cases, but for now expect some manual management of IP addresses and
sockets.

## Lattices

## Strata

