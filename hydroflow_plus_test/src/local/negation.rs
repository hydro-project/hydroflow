use hydroflow_plus::tokio::sync::mpsc::UnboundedSender;
use hydroflow_plus::*;
use stageleft::{q, Quoted, RuntimeData};

#[stageleft::entry]
pub fn test_difference<'a>(
    flow: FlowBuilder<'a, SingleProcessGraph>,
    output: RuntimeData<&'a UnboundedSender<u32>>,
    persist1: bool,
    persist2: bool,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let process = flow.process(&());

    let mut source = flow.source_iter(&process, q!(0..5));
    if persist1 {
        source = source.all_ticks();
    }

    let mut source2 = flow.source_iter(&process, q!(3..6));
    if persist2 {
        source2 = source2.all_ticks();
    }

    source.filter_not_in(source2).for_each(q!(|v| {
        output.send(v).unwrap();
    }));

    flow.extract().optimize_default()
}

#[stageleft::entry]
pub fn test_anti_join<'a>(
    flow: FlowBuilder<'a, SingleProcessGraph>,
    output: RuntimeData<&'a UnboundedSender<u32>>,
    persist1: bool,
    persist2: bool,
) -> impl Quoted<'a, Hydroflow<'a>> {
    let process = flow.process(&());

    let mut source = flow.source_iter(&process, q!(0..5)).map(q!(|v| (v, v)));
    if persist1 {
        source = source.all_ticks();
    }

    let mut source2 = flow.source_iter(&process, q!(3..6));
    if persist2 {
        source2 = source2.all_ticks();
    }

    // TODO(shadaj): inference fails without a for_each type annotation here
    source.anti_join(source2).for_each(q!(|v: (u32, u32)| {
        output.send(v.0).unwrap();
    }));

    flow.extract().optimize_default()
}

#[stageleft::runtime]
#[cfg(test)]
mod tests {
    use hydroflow_plus::assert_graphvis_snapshots;
    use hydroflow_plus::util::collect_ready;

    #[test]
    fn test_difference_tick_tick() {
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut flow = super::test_difference!(&out, false, false);
        assert_graphvis_snapshots!(flow);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[] as &[u32]);
    }

    #[test]
    fn test_difference_tick_static() {
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut flow = super::test_difference!(&out, false, true);
        assert_graphvis_snapshots!(flow);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[] as &[u32]);
    }

    #[test]
    fn test_difference_static_tick() {
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut flow = super::test_difference!(&out, true, false);
        assert_graphvis_snapshots!(flow);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);

        flow.run_tick();

        assert_eq!(
            &*collect_ready::<Vec<_>, _>(&mut out_recv),
            &[0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn test_difference_static_static() {
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut flow = super::test_difference!(&out, true, true);
        assert_graphvis_snapshots!(flow);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);
    }

    #[test]
    fn test_anti_join_tick_tick() {
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut flow = super::test_anti_join!(&out, false, false);
        assert_graphvis_snapshots!(flow);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[] as &[u32]);
    }

    #[test]
    fn test_anti_join_tick_static() {
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut flow = super::test_anti_join!(&out, false, true);
        assert_graphvis_snapshots!(flow);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[] as &[u32]);
    }

    #[test]
    fn test_anti_join_static_tick() {
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut flow = super::test_anti_join!(&out, true, false);
        assert_graphvis_snapshots!(flow);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);

        flow.run_tick();

        assert_eq!(
            &*collect_ready::<Vec<_>, _>(&mut out_recv),
            &[0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn test_anti_join_static_static() {
        let (out, mut out_recv) = hydroflow_plus::util::unbounded_channel();

        let mut flow = super::test_anti_join!(&out, true, true);
        assert_graphvis_snapshots!(flow);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);

        flow.run_tick();

        assert_eq!(&*collect_ready::<Vec<_>, _>(&mut out_recv), &[0, 1, 2]);
    }
}
