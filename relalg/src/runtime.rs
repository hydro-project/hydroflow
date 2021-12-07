#![allow(dead_code)]

use std::{cell::RefCell, rc::Rc};

use hydroflow::scheduled::{
    collections::Iter,
    ctx::{OutputPort, RecvCtx},
    handoff::VecHandoff,
    Hydroflow,
};

use crate::{Datum, RelExpr};

pub(crate) fn run_dataflow(r: RelExpr) -> Vec<Vec<Datum>> {
    let mut df = Hydroflow::new();

    let output_port = render_relational(&mut df, r);

    let output = Rc::new(RefCell::new(Vec::new()));
    let inner = output.clone();
    let sink = df.add_sink(move |_ctx, recv: &RecvCtx<VecHandoff<Vec<Datum>>>| {
        for v in recv.take_inner() {
            (*inner).borrow_mut().push(v);
        }
    });
    df.add_edge(output_port, sink);

    df.tick();

    let v = (*output).borrow();
    v.clone()
}

fn render_relational(df: &mut Hydroflow, r: RelExpr) -> OutputPort<VecHandoff<Vec<Datum>>> {
    match r {
        RelExpr::Values(mut v) => {
            // TODO: drip-feed data?
            let scope = Vec::new();
            df.add_source(move |_ctx, send| {
                send.give(Iter(
                    v.drain(..)
                        .map(|row| row.into_iter().map(|e| e.eval(&scope)).collect()),
                ));
            })
        }
        RelExpr::Filter(preds, v) => {
            let input = render_relational(df, *v);
            let (filter_in, filter_out) =
                df.add_inout(move |_ctx, recv: &RecvCtx<VecHandoff<Vec<Datum>>>, send| {
                    send.give(Iter(recv.take_inner().into_iter().filter(|row| {
                        preds.iter().all(|p| p.eval(row) == Datum::Bool(true))
                    })));
                });
            df.add_edge(input, filter_in);

            filter_out
        }
        RelExpr::Project(exprs, v) => {
            let input = render_relational(df, *v);
            let (project_in, project_out) =
                df.add_inout(move |_ctx, recv: &RecvCtx<VecHandoff<Vec<Datum>>>, send| {
                    send.give(Iter(
                        recv.take_inner()
                            .into_iter()
                            .map(|row| exprs.iter().map(|e| e.eval(&row)).collect()),
                    ));
                });
            df.add_edge(input, project_in);

            project_out
        }
    }
}
