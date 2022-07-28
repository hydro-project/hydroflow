use super::{Props, Spec};

pub trait PusheratorBuild {
    type SpecOut<PrevSpec>: Spec
    where
        PrevSpec: Spec;
    type PropsOut<PrevProps>: Props
    where
        PrevProps: Props;

    type ItemIn;

    fn next(&mut self, item: Self::ItemIn);
}

// pub struct DebugSink<T>
// where
//     T: std::fmt::Debug,
// {
//     _phantom: std::marker::PhantomData<fn(T)>,
// }
// impl<T> DebugSink<T>
// where
//     T: std::fmt::Debug,
// {
//     pub fn new() -> Self {
//         Self {
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
