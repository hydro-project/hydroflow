use crate::protocol::{Timestamp, Token};
use hydroflow::{hydroflow_syntax, scheduled::graph::Hydroflow};

use std::collections::VecDeque;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub(crate) fn rga_adjacency(
    input_recv: UnboundedReceiverStream<(Token, Timestamp)>,
    rga_send: UnboundedSender<(Token, Timestamp)>,
    list_send: UnboundedSender<(Timestamp, Timestamp)>,
) -> Hydroflow {
    hydroflow_syntax! {
        insertAfter = source_stream(input_recv) -> tee();

        // adjacency(parent:Timestamp, kids:VecDeque<Timestamp>, sibs:Vec<(Timestamp,Timestamp>)) tuples
        adjacency_sibs = insertAfter[adjacency]
          -> map(|(child, parent): (Token, Timestamp)| (parent, child.ts))
          -> group_by::<'tick, Timestamp, (VecDeque<Timestamp>, Vec<(Timestamp, Timestamp)>)>(
                    || (VecDeque::<Timestamp>::new(), Vec::<(Timestamp, Timestamp)>::new()),
                    |(kids, sibs): &mut (VecDeque<Timestamp>, Vec::<(Timestamp, Timestamp)>), val: Timestamp| {
                        // siblings (older, younger)
                        if !kids.is_empty() {
                            sibs.push((val, *kids.back().unwrap()));
                        }
                        kids.push_back(val);
            })
          -> map(|(parent, (kids, sibs))| ((parent, kids), sibs)) -> unzip();
        adjacency = adjacency_sibs[0]  -> tee();
        siblings = adjacency_sibs[1] -> flatten();
        leaves = difference();
        split = insertAfter[diff] -> unzip();
        split[0] -> map(|child| child.ts) -> [pos]leaves;
        split[1] -> [neg]leaves;

        // nextSiblingAnc
        nextSiblingAnc = merge() -> tee();
        // siblings(bigger, smaller)
        siblings -> nextSiblingAnc;
        // lastChild(parent, lastChild)
        lastChild = adjacency[lastChild] -> map(|(parent, kids): (Timestamp, VecDeque<Timestamp>)| (parent, *kids.front().unwrap()));
        upEdge = join() -> map(|(_parent, (last_child, next_sib)): (Timestamp, (Timestamp, Timestamp))| (last_child, next_sib));
        lastChild -> [0]upEdge;
        nextSiblingAnc -> [1]upEdge;
        upEdge -> nextSiblingAnc;

        // nextElem
        nextElem = merge();
        adjacency[nextElem] -> map(|(parent, kids): (Timestamp, VecDeque<Timestamp>)| (parent, *kids.back().unwrap())) -> nextElem;
        // nextElem(Prev,Next) :- isListElem(Prev), !hasChild (Prev), nextSiblingAnc(Prev,Next)
        ne_join = join() -> map(|(prev, (_, next))| (prev, next));
        leaves -> map(|prev| (prev, ())) -> [0]ne_join;
        nextSiblingAnc -> [1]ne_join;
        ne_join -> nextElem;

        insertAfter[print] -> for_each(|(c, p): (Token, Timestamp)| rga_send.send((c, p)).unwrap());
        nextElem -> for_each(|(first, second)| list_send.send((first, second)).unwrap());
    }
}

// // stack subroutine:
// // - accepts vecDeque batches to insert
// // - pops one element per tick
// // each batch is pushed onto the stack in sort order
// // so that pops are in the correct order (largest id first)
// stack_buf = merge() -> reduce(|mut accum: VecDeque<Timestamp>, mut v: VecDeque<Timestamp>| {
//     accum.append(&mut v);
//     accum
// });
// stack = merge()
//         -> fold(VecDeque::new(), |mut accum: VecDeque<Timestamp>, mut v: VecDeque<Timestamp>| {
//             v.make_contiguous().sort();
//             v.iter().for_each(|e| accum.push_front(*e));
//             accum
//         });
// // stack -> for_each(|v: VecDeque<Timestamp>| println!("stack: {:?}", v));
// stack_buf -> next_tick() -> stack;
// pop_stack = stack -> map(|mut q: VecDeque<Timestamp>| { let e = q.pop_front().unwrap(); (q, e)}) -> unzip();
// pop_stack[0] -> filter(|v: &VecDeque<Timestamp>| v.len() > 0) -> stack_buf;
// visit = pop_stack[1] -> tee();

// // initialize output by pushing root id on the stack
//         root = insertAfter -> map(|(_child, parent)| parent) -> reduce(|accum: Timestamp, elem: Timestamp| if accum < elem {accum} else {elem} )
// root -> map(|v| { let mut d = VecDeque::new(); d.push_front(v); d}) -> stack;

// // "visit" a node by looking up its children and pushing them onto the stack
// children = join::<'tick, 'static>() -> map(|(_n, (_, kids))| kids);
// visit -> map(|n| (n, ())) -> [0]children;
// adjacency -> [1]children;
// children -> stack_buf;

// labels = insertAfter -> map(|(c, _p): (Token, Timestamp)| (c.id, c.value));
// visit_labels = join::<'tick, 'static>() -> map(|(_id, ((), label))| label);
// visit -> map(|n| (n, ())) -> [0]visit_labels;
// labels -> [1]visit_labels;
// visit_labels -> for_each(|e| print!("{}", e));
