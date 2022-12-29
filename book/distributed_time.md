# Distributed Time
Hydroflow does not embed a particular notion of distributed time, but instead provides primitives for 
you to implement one of many possible distributed time models.

## Lamport Time
Lamport's paper on [Time, Clocks, and the Ordering of Events in a Distributed System](https://lamport.azurewebsites.net/pubs/time-clocks.pdf) provides a classical definition of time in a distributed system. In that paper, each "process" has a sequential clock (a so-called [Lamport Clock](https://en.wikipedia.org/wiki/Lamport_timestamp)). The clock in a process advances by one unit of time for each "event" that the process observes. Between events, the clock is frozen and the process does local computation.

In addition, to maintain Lamport Clocks, each process stamps its outbound messages with the current clock value, and begins each "tick" by advancing its local clock to the maximum of 

- its current clock value and 
- the largest clock value of any message it received. 

This has the desirable property of respecting a reasonable distributed "happens-before" relation. 
That is, Lamport timestamps track not only the order of events on a single node, they also ensure that the timestamps on events reflect distributed ordering. Suppose that node `source` wants to send a message to node `dest`, and node source has current clock value *T_source*. The events that precede that message on node `source` have smaller timestamps. In addition, consider an event at node `dest` that follows the receipt of that message. That event must have a timestamp greater than *T_source*, and hence all the events on node `source` that preceded the sending of the message have lower timestamps than the events on node `dest` following the receipt of the message. This is the distributed "happens-before" relation.

## Hydroflow Time
As a built-in primitive, Hydroflow defines time only for a single transducer, as a sequence of ticks. Each tick captures Hydroflow's notion of an "event" as the atomic ingestion of a *batch of events*, followed by a fixpoint computation over that batch. *TODO: is it possible to limit batch size to 1?* 

Thus the main difference between Hydroflow events and Lamport events are:

1. Hydroflow treats the ingestion of multiple inbound events as a single timestep.
2. Hydroflow *requires* a fixpoint computation to complete between events.

(Technically, Hydroflow can be seen as a form of [transducer](https://en.wikipedia.org/wiki/Finite-state_transducer).)

In addition, the built-in clock primitive in Hydroflow always advances sequentially and cannot skip a tick like the Lamport clock does.

## Implementing Distributed Time in Hydroflow

Although Lamport clocks are not built into Hydroflow, it is straightforward to implement them in Hydroflow. Alternatively, one can implement more sophisticated distributed clocks like [vector clocks](https://en.wikipedia.org/wiki/Vector_clock) in Hydroflow instead. By leaving this decision to the user, Hydroflow can be used in a variety of distributed time models.

*TODO: add example of implementing Lamport clocks?*.