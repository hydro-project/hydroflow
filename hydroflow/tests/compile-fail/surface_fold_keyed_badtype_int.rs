fn main() {
    let mut df = hydroflow::dfir_syntax! {
        source_iter(0..1)
            -> reduce_keyed(|old: &mut u32, val: u32| { *old += val; })
            -> for_each(std::mem::drop);
    };
    df.run_available();
}
