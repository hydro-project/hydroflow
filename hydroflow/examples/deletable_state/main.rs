use hydroflow::hydroflow_syntax;

pub fn main() {
    {
        println!("persist() -> fold()");
        hydroflow_syntax! {
            source_iter([2, 3, 5, 7])
                -> persist()
                -> fold::<'tick>(0, |a, b| a + b)
                -> for_each(|string| println!("{:?}", string));
        }
        .run_available();

        println!("persist() -> fold() => fold<'static>()");
        hydroflow_syntax! {
            source_iter([2, 3, 5, 7])
                -> fold::<'static>(0, |a, b| a + b)
                -> for_each(|string| println!("{:?}", string));
        }
        .run_available();
    }

    {
        println!("persist() -> fold_keyed()");
        hydroflow_syntax! {
            source_iter([(0, 2), (0, 3), (1, 5), (0, 7)])
                -> persist()
                -> fold_keyed::<'tick>(|| 0, |a: &mut i32, b| *a += b)
                -> for_each(|string| println!("{:?}", string));
        }
        .run_available();

        println!("persist() -> fold_keyed<'static>()");
        hydroflow_syntax! {
            source_iter([(0, 2), (0, 3), (1, 5), (0, 7)])
                -> fold_keyed::<'static>(|| 0, |a: &mut i32, b| *a += b)
                -> for_each(|string| println!("{:?}", string));
        }
        .run_available();
    }

    {
        use hydroflow::util::Persistence::*;

        println!("persist_mut() -> fold()");
        hydroflow_syntax! {
            source_iter([Delete(2), Persist(2), Persist(3), Persist(5), Persist(7), Delete(2)])
                -> persist_mut()
                -> fold::<'tick>(0, |a, b| a + b)
                -> for_each(|string| println!("{:?}", string));
        }
        .run_available();

        println!("persist_mut() -> fold() => todo!()");
        println!("cannot implement without inverses");
    }

    {
        use hydroflow::util::PersistenceKeyed::*;

        println!("persist_mut_keyed() -> fold_keyed()");
        hydroflow_syntax! {
            source_iter([Delete(0), Persist(0, 2), Persist(0, 3), Persist(1, 5), Persist(0, 7), Delete(0)])
                -> persist_mut_keyed()
                -> fold_keyed::<'tick>(|| 0, |a: &mut i32, b| *a += b)
                -> for_each(|string| println!("{:?}", string));
        }
        .run_available();

        println!("persist_mut_keyed() -> fold_keyed() => fold_keyed<'mutable>()");
        hydroflow_syntax! {
            source_iter([Delete(0), Persist(0, 2), Persist(0, 3), Persist(1, 5), Persist(0, 7), Delete(0)])
                -> fold_keyed::<'mutable>(|| 0, |a: &mut i32, b| *a += b)
                -> for_each(|string| println!("{:?}", string));
        }
        .run_available();
    }
}
