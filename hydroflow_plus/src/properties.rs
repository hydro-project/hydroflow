use std::collections::HashSet;

use stageleft::*;

use crate::ir::{HfPlusLeaf, HfPlusNode, SeenTees};

/// schema : table per algebraic property with the list of expressions that satisfy the property
/// interface: can "tag" an expression with a property and it will add it to that table
/// can also run a check to see if an expression satisfies a property
#[derive(Default)]
pub struct PropertyDatabase {
    commutative: HashSet<syn::Expr>,
}

#[allow(dead_code)]
// allows us to convert the hydroflow datatype for folds to a binary operation for the algebra property tests
fn convert_hf_to_binary<I, A: Default, F: Fn(&mut A, I)>(f: F) -> impl Fn(I, I) -> A {
    move |a, b| {
        let mut acc = Default::default();
        f(&mut acc, a);
        f(&mut acc, b);
        acc
    }
}

impl PropertyDatabase {
    pub fn add_commutative_tag<'a, I, A, F: Fn(&mut A, I), Q: Quoted<'a, F> + Clone>(
        &mut self,
        expr: Q,
    ) -> Q {
        let expr_clone = expr.clone();
        self.commutative.insert(expr_clone.splice());
        expr
    }

    pub fn is_tagged_commutative(&self, expr: &syn::Expr) -> bool {
        self.commutative.contains(expr)
    }
}

// Dataflow graph optimization rewrite rules based on algebraic property tags
// TODO add a test that verifies the space of possible graphs after rewrites is correct for each property

fn properties_optimize_node(
    node: HfPlusNode,
    db: &PropertyDatabase,
    seen_tees: &mut SeenTees,
) -> HfPlusNode {
    match node.transform_children(
        |node, seen_tees| properties_optimize_node(node, db, seen_tees),
        seen_tees,
    ) {
        HfPlusNode::ReduceKeyed { f, input } if db.is_tagged_commutative(&f.0) => {
            dbg!("IDENTIFIED COMMUTATIVE OPTIMIZATION for {:?}", &f);
            HfPlusNode::ReduceKeyed { f, input }
        }
        o => o,
    }
}

pub fn properties_optimize(ir: Vec<HfPlusLeaf>, db: &PropertyDatabase) -> Vec<HfPlusLeaf> {
    let mut seen_tees = Default::default();
    ir.into_iter()
        .map(|l| {
            l.transform_children(
                |node, seen_tees| properties_optimize_node(node, db, seen_tees),
                &mut seen_tees,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_property_database() {
        let mut db = PropertyDatabase::default();

        assert!(!db.is_tagged_commutative(&(q!(|a: &mut i32, b: i32| *a += b).splice())));

        let _ = db.add_commutative_tag(q!(|a: &mut i32, b: i32| *a += b));

        assert!(db.is_tagged_commutative(&(q!(|a: &mut i32, b: i32| *a += b).splice())));
    }

    #[test]
    fn test_property_optimized() {
        let flow = FlowBuilder::<SingleProcessGraph>::new();
        let mut database = PropertyDatabase::default();

        let process = flow.process(&());

        let counter_func = q!(|count: &mut i32, _| *count += 1);
        let _ = database.add_commutative_tag(counter_func);

        flow.source_iter(&process, q!(vec![]))
            .map(q!(|string: String| (string, ())))
            .fold_keyed(q!(|| 0), counter_func)
            .for_each(q!(|(string, count)| println!("{}: {}", string, count)));

        let built = flow
            .extract()
            .optimize_with(|ir| properties_optimize(ir, &database))
            .with_default_optimize();

        insta::assert_debug_snapshot!(built.ir());
    }
}
