use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow::hydroflow_syntax! {
        source_iter(["hello", "world"])
            -> group_by::<'tick, &str, usize>(String::new, |old: &mut _, val| {
                *old += val;
            })
            -> for_each(|(k, v)| println!("({:?}, {:?})", k, v));
    };
    df.run_available();
}
