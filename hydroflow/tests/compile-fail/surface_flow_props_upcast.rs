pub fn main() {
    let mut hf = hydroflow::hydroflow_syntax! {
        source_iter_delta((0..10).map(SetUnionSingletonSet::new_from))
            -> cast(None)
            -> map(|SetUnion(SingletonSet(x))| 10 * x)
            -> cast(Some(Delta))
            -> for_each(|x| println!("seq {:?}", x));
    };
    hf.run_available();
}
