use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;

use crate::ir::*;

fn persist_pullup_node<'a>(
    node: &mut HfPlusNode<'a>,
    persist_pulled_tees: &mut HashSet<*const RefCell<HfPlusNode<'a>>>,
) {
    match node {
        HfPlusNode::Unpersist(box HfPlusNode::Persist(_)) => {
            if let HfPlusNode::Unpersist(box HfPlusNode::Persist(box behind_persist)) =
                std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node = behind_persist;
            } else {
                unreachable!()
            }
        }

        HfPlusNode::Delta(box HfPlusNode::Persist(_)) => {
            if let HfPlusNode::Delta(box HfPlusNode::Persist(box behind_persist)) =
                std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node = behind_persist;
            } else {
                unreachable!()
            }
        }

        HfPlusNode::Tee { inner } => {
            if persist_pulled_tees.contains(&(inner.as_ref() as *const RefCell<HfPlusNode<'a>>)) {
                *node = HfPlusNode::Persist(Box::new(HfPlusNode::Tee {
                    inner: inner.clone(),
                }));
            } else {
                let inner_borrow = inner.borrow();
                if let HfPlusNode::Persist(_) = inner_borrow.deref() {
                    drop(inner_borrow);
                    persist_pulled_tees.insert(inner.as_ref() as *const RefCell<HfPlusNode<'a>>);
                    if let HfPlusNode::Persist(box behind_persist) =
                        inner.replace(HfPlusNode::Placeholder)
                    {
                        *inner.borrow_mut() = behind_persist;
                    } else {
                        unreachable!()
                    }

                    *node = HfPlusNode::Persist(Box::new(HfPlusNode::Tee {
                        inner: inner.clone(),
                    }));
                }
            }
        }

        HfPlusNode::Map {
            f: _,
            input: box HfPlusNode::Persist(_),
        } => {
            if let HfPlusNode::Map {
                f,
                input: box HfPlusNode::Persist(behind_persist),
            } = std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node = HfPlusNode::Persist(Box::new(HfPlusNode::Map {
                    f,
                    input: behind_persist,
                }));
            } else {
                unreachable!()
            }
        }

        HfPlusNode::FlatMap {
            f: _,
            input: box HfPlusNode::Persist(_),
        } => {
            if let HfPlusNode::FlatMap {
                f,
                input: box HfPlusNode::Persist(behind_persist),
            } = std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node = HfPlusNode::Persist(Box::new(HfPlusNode::FlatMap {
                    f,
                    input: behind_persist,
                }));
            } else {
                unreachable!()
            }
        }

        HfPlusNode::Filter {
            f: _,
            input: box HfPlusNode::Persist(_),
        } => {
            if let HfPlusNode::Filter {
                f,
                input: box HfPlusNode::Persist(behind_persist),
            } = std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node = HfPlusNode::Persist(Box::new(HfPlusNode::Filter {
                    f,
                    input: behind_persist,
                }));
            } else {
                unreachable!()
            }
        }

        HfPlusNode::Network {
            input: box HfPlusNode::Persist(_),
            ..
        } => {
            if let HfPlusNode::Network {
                from_location,
                to_location,
                serialize_pipeline,
                instantiate_fn,
                deserialize_pipeline,
                input: box HfPlusNode::Persist(behind_persist),
            } = std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node = HfPlusNode::Persist(Box::new(HfPlusNode::Network {
                    from_location,
                    to_location,
                    serialize_pipeline,
                    instantiate_fn,
                    deserialize_pipeline,
                    input: behind_persist,
                }));
            } else {
                unreachable!()
            }
        }

        HfPlusNode::Union(box HfPlusNode::Persist(_), box HfPlusNode::Persist(_)) => {
            if let HfPlusNode::Union(
                box HfPlusNode::Persist(left),
                box HfPlusNode::Persist(right),
            ) = std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node = HfPlusNode::Persist(Box::new(HfPlusNode::Union(left, right)));
            } else {
                unreachable!()
            }
        }

        HfPlusNode::CrossProduct(box HfPlusNode::Persist(_), box HfPlusNode::Persist(_)) => {
            if let HfPlusNode::CrossProduct(
                box HfPlusNode::Persist(left),
                box HfPlusNode::Persist(right),
            ) = std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node = HfPlusNode::Persist(Box::new(HfPlusNode::Delta(Box::new(
                    HfPlusNode::CrossProduct(
                        Box::new(HfPlusNode::Persist(left)),
                        Box::new(HfPlusNode::Persist(right)),
                    ),
                ))));
            } else {
                unreachable!()
            }
        }

        HfPlusNode::Join(box HfPlusNode::Persist(_), box HfPlusNode::Persist(_)) => {
            if let HfPlusNode::Join(box HfPlusNode::Persist(left), box HfPlusNode::Persist(right)) =
                std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node =
                    HfPlusNode::Persist(Box::new(HfPlusNode::Delta(Box::new(HfPlusNode::Join(
                        Box::new(HfPlusNode::Persist(left)),
                        Box::new(HfPlusNode::Persist(right)),
                    )))));
            } else {
                unreachable!()
            }
        }

        HfPlusNode::Unique(box HfPlusNode::Persist(_)) => {
            if let HfPlusNode::Unique(box HfPlusNode::Persist(inner)) =
                std::mem::replace(node, HfPlusNode::Placeholder)
            {
                *node = HfPlusNode::Persist(Box::new(HfPlusNode::Delta(Box::new(
                    HfPlusNode::Unique(Box::new(HfPlusNode::Persist(inner))),
                ))));
            } else {
                unreachable!()
            }
        }

        _ => {}
    }
}

pub fn persist_pullup(ir: Vec<HfPlusLeaf>) -> Vec<HfPlusLeaf> {
    let mut seen_tees = Default::default();
    let mut persist_pulled_tees = Default::default();
    ir.into_iter()
        .map(|l| {
            l.transform_children(
                |n, s| n.transform_bottom_up(persist_pullup_node, s, &mut persist_pulled_tees),
                &mut seen_tees,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use stageleft::*;

    use crate::deploy::MultiGraph;

    #[test]
    fn persist_pullup_through_map() {
        let flow = crate::builder::FlowBuilder::new();
        let process = flow.process::<()>();

        flow.source_iter(&process, q!(0..10))
            .map(q!(|v| v + 1))
            .for_each(q!(|n| println!("{}", n)));

        let built = flow.finalize();

        insta::assert_debug_snapshot!(built.ir());

        let optimized = built.optimize_with(super::persist_pullup);

        insta::assert_debug_snapshot!(optimized.ir());
        for (id, graph) in optimized.compile_no_network::<MultiGraph>().hydroflow_ir() {
            insta::with_settings!({snapshot_suffix => format!("surface_graph_{id}")}, {
                insta::assert_snapshot!(graph.surface_syntax_string());
            });
        }
    }

    #[test]
    fn persist_pullup_behind_tee() {
        let flow = crate::builder::FlowBuilder::new();
        let process = flow.process::<()>();

        let before_tee = flow.source_iter(&process, q!(0..10)).tick_batch().persist();

        before_tee
            .clone()
            .map(q!(|v| v + 1))
            .all_ticks()
            .for_each(q!(|n| println!("{}", n)));

        before_tee
            .clone()
            .map(q!(|v| v + 1))
            .all_ticks()
            .for_each(q!(|n| println!("{}", n)));

        let built = flow.finalize();

        insta::assert_debug_snapshot!(built.ir());

        let optimized = built.optimize_with(super::persist_pullup);

        insta::assert_debug_snapshot!(optimized.ir());

        for (id, graph) in optimized.compile_no_network::<MultiGraph>().hydroflow_ir() {
            insta::with_settings!({snapshot_suffix => format!("surface_graph_{id}")}, {
                insta::assert_snapshot!(graph.surface_syntax_string());
            });
        }
    }
}
