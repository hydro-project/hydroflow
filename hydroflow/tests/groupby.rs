use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;

use itertools::Itertools;

use hydroflow::lang::collections::Iter;
use hydroflow::scheduled::graph::Hydroflow;
use hydroflow::scheduled::graph_ext::GraphExt;
use hydroflow::scheduled::handoff::VecHandoff;

#[derive(Clone, Copy, Debug)]
pub struct Employee {
    pub name: &'static str,
    pub department: &'static str,
    pub salary: u32,
}
const BATCH_A: &[Employee] = &[
    Employee {
        name: "Davis",
        department: "hr",
        salary: 7000,
    },
    Employee {
        name: "Megan",
        department: "accounting",
        salary: 6500,
    },
    Employee {
        name: "Zach",
        department: "sales",
        salary: 8000,
    },
    Employee {
        name: "Pierce",
        department: "accounting",
        salary: 7100,
    },
    Employee {
        name: "Alexa",
        department: "sales",
        salary: 9000,
    },
];

const BATCH_B: &[Employee] = &[
    Employee {
        name: "Joanne",
        department: "engineering",
        salary: 6800,
    },
    Employee {
        name: "Suzanne",
        department: "hr",
        salary: 5900,
    },
    Employee {
        name: "Raul",
        department: "engineering",
        salary: 12000,
    },
    Employee {
        name: "Buffy",
        department: "sales",
        salary: 8800,
    },
    Employee {
        name: "Chaitali",
        department: "accounting",
        salary: 6900,
    },
];

const BATCH_C: &[Employee] = &[
    Employee {
        name: "Rene",
        department: "engineering",
        salary: 7000,
    },
    Employee {
        name: "Miles",
        department: "hr",
        salary: 6100,
    },
    Employee {
        name: "Phillip",
        department: "sales",
        salary: 5800,
    },
];

/// Basic monotonic threshold: find all departments with total salary at least 20_000.
/// Uses the core API.
/// SQL: SELECT department FROM employees WHERE 20_000 <= SUM(salary) GROUP BY department
#[test]
fn groupby_monotonic_core() {
    let mut hf = Hydroflow::new();

    let (source_send, source_recv) = hf.make_edge::<_, VecHandoff<Employee>>("source handoff");
    let input = hf.add_input("source", source_send);

    let (sink_send, sink_recv) = hf.make_edge::<_, VecHandoff<&'static str>>("sink handoff");

    let mut groups = HashMap::<&'static str, u32>::new();
    hf.add_subgraph_in_out(
        "group by",
        source_recv,
        sink_send,
        move |_ctx, recv, send| {
            for employee in recv.take_inner() {
                let cost = groups.entry(employee.department).or_default();
                let old_cost = *cost;
                *cost += employee.salary;
                if old_cost < 20_000 && 20_000 <= *cost {
                    send.give(Some(employee.department));
                }
            }
        },
    );

    let output = <Rc<RefCell<Vec<&'static str>>>>::default();
    let output_ref = output.clone();
    hf.add_subgraph_sink("sink", sink_recv, move |_ctx, recv| {
        for v in recv.take_inner().into_iter() {
            output_ref.borrow_mut().push(v);
        }
    });

    input.give(Iter(BATCH_A.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(0, output.borrow().len());

    input.give(Iter(BATCH_B.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(&["sales", "accounting"], &**output.borrow());

    input.give(Iter(BATCH_C.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(&["sales", "accounting", "engineering"], &**output.borrow());
}

/// Basic monotonic threshold: release a value once after it has been seen three times.
/// Uses the surface (builder) API.
/// SQL: SELECT department FROM employees WHERE 20_000 <= SUM(salary) GROUP BY department
#[test]
fn groupby_monotonic_surface() {
    use hydroflow::builder::prelude::*;

    let mut hf_builder = HydroflowBuilder::new();
    let (input, source_recv) = hf_builder.add_channel_input::<_, _, VecHandoff<Employee>>("source");

    let output = <Rc<RefCell<Vec<&'static str>>>>::default();
    let output_ref = output.clone();

    hf_builder.add_subgraph(
        "main",
        source_recv
            .flatten()
            .map_scan(HashMap::<&'static str, u32>::new(), |groups, employee| {
                let count = groups.entry(employee.department).or_default();
                let old_count = *count;
                *count += employee.salary;
                (employee.department, old_count, *count)
            })
            .filter_map(|(department, old_count, count)| {
                if old_count < 20_000 && 20_000 <= count {
                    Some(department)
                } else {
                    None
                }
            })
            .pull_to_push()
            .for_each(move |item| output_ref.borrow_mut().push(item)),
    );

    let mut hf = hf_builder.build();

    input.give(Iter(BATCH_A.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(0, output.borrow().len());

    input.give(Iter(BATCH_B.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(&["sales", "accounting"], &**output.borrow());

    input.give(Iter(BATCH_C.iter().cloned()));
    input.flush();
    hf.tick();
    assert_eq!(&["sales", "accounting", "engineering"], &**output.borrow());
}

/// Non-monotonic barrier. Per epoch, for each department find the highest paid employee.
/// Takes in BATCH_A in the first epoch, then BATCH_B *and* BATCH_C in the second epoch.
/// SQL (per batch): SELECT department, name, salary FROM employees WHERE salary = MAX(salary) GROUP BY department
#[test]
fn groupby_nonmon_surface() {
    use hydroflow::builder::prelude::*;

    let mut hf_builder = HydroflowBuilder::new();
    let (input, source_recv) = hf_builder.add_channel_input::<_, _, VecHandoff<Employee>>("source");

    let (a_send, a_recv) = hf_builder.make_edge::<_, VecHandoff<Employee>, _>("names A-M");
    let (z_send, z_recv) = hf_builder.make_edge::<_, VecHandoff<Employee>, _>("names N-Z");
    let (stratum_boundary_send, stratum_boundary_recv) =
        hf_builder.make_edge::<_, VecHandoff<Employee>, _>("names N-Z");

    // Make output first, to mess with scheduler order.
    let output = <Rc<RefCell<BinaryHeap<_>>>>::default();
    let output_ref = output.clone();
    hf_builder.add_subgraph_stratified(
        "find median",
        1,
        stratum_boundary_recv
            .flat_map(|batch| {
                batch
                    .into_iter()
                    .into_grouping_map_by(|employee| employee.department)
                    .max_by_key(|_department, employee| employee.salary)
            })
            .pull_to_push()
            .map(|(department, employee)| (department, employee.name, employee.salary))
            .for_each(move |val| output_ref.borrow_mut().push(val)),
    );

    // Partition then re-merge names to make graph more interesting.
    // Want to have multiple compiled components to test scheduler.
    hf_builder.add_subgraph_stratified(
        "split",
        0,
        source_recv.flatten().pull_to_push().partition(
            |employee| employee.name < "N",
            hf_builder.start_tee().map(Some).push_to(a_send),
            hf_builder.start_tee().map(Some).push_to(z_send),
        ),
    );
    hf_builder.add_subgraph_stratified(
        "merge",
        0,
        a_recv
            .flatten()
            .chain(z_recv.flatten())
            .pull_to_push()
            .map(Some)
            .push_to(stratum_boundary_send),
    );

    let mut hf = hf_builder.build();

    // Give BATCH_A and cross barrier to run next stratum.
    input.give(Iter(BATCH_A.iter().cloned()));
    input.flush();
    hf.tick_stratum();
    assert_eq!((0, 0), (hf.current_epoch(), hf.current_stratum()));

    assert_eq!(0, output.borrow().len());

    hf.tick();
    assert_eq!((1, 1), (hf.current_epoch(), hf.current_stratum()));

    assert_eq!(
        &[
            ("accounting", "Pierce", 7100),
            ("hr", "Davis", 7000),
            ("sales", "Alexa", 9000),
        ],
        &*std::mem::take(&mut *output.borrow_mut()).into_sorted_vec(),
    );

    // Give BATCH_B but only run this stratum.
    input.give(Iter(BATCH_B.iter().cloned()));
    input.flush();

    hf.tick_stratum();
    assert_eq!((1, 1), (hf.current_epoch(), hf.current_stratum()));

    // Give BATCH_C and run all to completion.
    input.give(Iter(BATCH_C.iter().cloned()));
    input.flush();

    hf.tick();
    assert_eq!((3, 1), (hf.current_epoch(), hf.current_stratum()));

    // Second batch has 7+3 = 10 items.
    assert_eq!(
        &[
            ("accounting", "Chaitali", 6900),
            ("engineering", "Raul", 12000),
            ("hr", "Miles", 6100),
            ("sales", "Buffy", 8800),
        ],
        &*std::mem::take(&mut *output.borrow_mut()).into_sorted_vec(),
    );
    assert_eq!(false, hf.next_stratum());
}
