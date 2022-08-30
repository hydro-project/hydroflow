use super::Props;

pub trait PullSpec {
    type Props: Props;
}

pub trait PushSpec {
    type Props<PrevProps: Props>;
}

macro_rules! spec_impl {
    (
        PullSpec for $($struct:ident::)* <$iname:ident $(,$gname:ident)*>
    ) => {
        impl<$iname $(,$gname)*> PullSpec for $($struct::)* <$iname $(,$gname)*>
        where
            $iname: PullSpec,
        {
            type Props = $iname::Props;
        }
    };
    (
        PushSpec for $($struct:ident::)* <$iname:ident $(,$gname:ident)*>
    ) => {
        impl<$iname $(,$gname)*> PushSpec for $($struct::)* <$iname $(,$gname)*>
        where
            $iname: PushSpec,
        {
            type Props<PrevProps: Props> = $iname::Props<PrevProps>;
        }
    };
}

spec_impl!(PullSpec for std::iter::Cloned::<I>);
spec_impl!(PullSpec for std::iter::Copied::<I>);
spec_impl!(PullSpec for std::iter::Filter::<I, P>);
spec_impl!(PullSpec for std::iter::FilterMap::<I, F>);
spec_impl!(PullSpec for std::iter::Inspect::<I, F>);
spec_impl!(PullSpec for std::iter::Map::<I, F>);

impl<I, U, F> PullSpec for std::iter::FlatMap<I, U, F>
where
    I: Iterator + PullSpec,
    U: IntoIterator,
    F: FnMut(I::Item) -> U,
{
    type Props = I::Props;
}

impl<I> PullSpec for std::iter::Flatten<I>
where
    I: Iterator + PullSpec,
    <I as Iterator>::Item: IntoIterator,
{
    type Props = I::Props;
}

spec_impl!(PushSpec for pusherator::filter_map::FilterMap::<Next, Func, In>);
spec_impl!(PushSpec for pusherator::filter::Filter::<Next, Func>);
spec_impl!(PushSpec for pusherator::flatten::Flatten::<Next, In>);
spec_impl!(PushSpec for pusherator::for_each::ForEach::<Func, In>);
spec_impl!(PushSpec for pusherator::map::Map::<Next, Func, In>);

impl<Next1, Next2, Func> PushSpec for pusherator::partition::Partition<Next1, Next2, Func>
where
    Next1: PushSpec,
    Next2: PushSpec,
{
    type Props<PrevProps: Props> = (Next1::Props<PrevProps>, Next2::Props<PrevProps>);
}

impl<Next1, Next2> PushSpec for pusherator::tee::Tee<Next1, Next2>
where
    Next1: PushSpec,
    Next2: PushSpec,
{
    type Props<PrevProps: Props> = (Next1::Props<PrevProps>, Next2::Props<PrevProps>);
}
