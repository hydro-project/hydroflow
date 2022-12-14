use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        source_iter(0..1)
            -> group_by(|| 0, |old: &mut u32, val: u32| *old += val)
            -> for_each(std::mem::drop)
    };
    df.run_available();
}
