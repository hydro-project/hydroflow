//! Hydroflow's Surface API.
//!
//! ## Internal Documentation
//!
//! Due to the limitations of type-level programming in Rust, this code is
//! "baklava" code containing lot of layers. Each layer does one thing, then
//! constructs the next layer(s) down. This table describes what each layer
//! does and is named. Layers are listed starting from the highest
//! (user-facing API) layer and ending with the lowest (code-running) layer.
//!
//! ### Layer Structure
//! ```text
//!               (A) Surface API
//!         (B) (Push Surface Reversed*)
//!                  /            \
//!                 /              \
//!        (C) Connector   (D) Subgraph Builder
//!                                  |
//!                       (E) Iterator/Pusherator
//! ```
//! <sup>*Only used with `Push` to reverse the ownership direction.</sup>
//!
//! ### Layer Descriptions
//! <table>
//! <tr>
//!     <th rowspan="2">Layer</th>
//!     <th rowspan="2">Purpose</th>
//!     <th colspan="2">Naming</th>
//! </tr>
//! <tr>
//!    <th>Pull</th>
//!    <th>Push</th>
//! </tr>
//! <tr>
//!     <td>(A) The Surface API</td>
//!     <td rowspan="2">
//!         &bull; Presents a clean functional-chaining API for users.<br>
//!         &bull; Consumed to simultaneously create a (C) connector and (D) builder.<br>
//!         &bull; <strong>Push Only</strong>: Extra layer needed to reverse ownership order.
//!     </td>
//!     <td><code>PullSurface</code></td>
//!     <td><code>PushSurface</code></td>
//! </tr>
//! <tr>
//!     <td>(B) Push Surface Reversed</td>
//!     <td>N/A</td>
//!     <td><code>PushSurfaceReversed</code></td>
//! </tr>
//! <tr>
//!     <td>(C) Connectors</td>
//!     <td>
//!         &bull; Connects <code>OutputPort</code>s and <code>InputPort</code>s, splits type lists in order to do so.<br>
//!         &bull; Does not go to any lower layers.<br>
//!         &bull; Uses the input/output <code>HandoffList</code> variadic type.
//!     </td>
//!     <td><code>PullConnect</code></td>
//!     <td><code>PushConnect</code></td>
//! </tr>
//! <tr>
//!     <td>(D) Subgraph Builders</td>
//!     <td>
//!         &bull; On each subgraph invocation, constructs the (E) iterators and pivot which will be run.<br>
//!         &bull; Is owned by the subgraph task, lends closures to (E) iterators.<br>
//!         &bull; Uses the input/output <code>HandoffList</code> variadic type.
//!     </td>
//!     <td><code>PullBuild</code></td>
//!     <td><code>PushBuild</code></td>
//! </tr>
//! <tr>
//!     <td>(E) Iterators</td>
//!     <td>
//!         &bull; Runs code on individual dataflow elements, in the case of dataflow.<br>
//!         &bull; In the future, will correspond to semilattice morphisms alternatively.
//!     </td>
//!     <td><code>std::iter::Iterator</code></td>
//!     <td><code>crate::compiled::Pusherator</code></td>
//! </tr>
//! </table>
//!
//! ### How the code actually runs
//!
//! The layers are used in [HydroflowBuilder::add_subgraph]. The method
//! receives a pivot with `PullSurface` and `PushSurfaceReversed` halves. Then
//! `into_parts()` splits them into the build half and the connector half.
//! The build half is used to create the subgraph which gives back
//! input/output ports. Then the connector half is used to connect up those
//! ports.

pub mod build;
pub mod connect;
pub mod surface;

mod hydroflow_builder;
pub use hydroflow_builder::HydroflowBuilder;

/// Prelude to quick-import items needed for using the Surface API.
///
/// Usage:
/// ```rust
/// use hydroflow::builder::prelude::*;
/// ```
pub mod prelude {
    pub use super::surface::{BaseSurface, PullSurface, PushSurface};
    pub use super::HydroflowBuilder;
}

#[test]
fn test_teeing() {
    use crate::scheduled::handoff::VecHandoff;
    use prelude::*;

    use std::cell::RefCell;
    use std::rc::Rc;

    let mut builder = HydroflowBuilder::default();
    let (ingress_send, ingress) = builder.add_channel_input::<Option<usize>, VecHandoff<_>>();

    let output_evn: Rc<RefCell<Vec<usize>>> = Default::default();
    let output_odd: Rc<RefCell<Vec<usize>>> = Default::default();

    let output_evn_take = Rc::clone(&output_evn);
    let output_odd_take = Rc::clone(&output_odd);

    let sg = ingress
        .flat_map(std::convert::identity)
        .flat_map(|x| [11 * x, x])
        .pivot()
        .tee(
            builder
                .start_tee()
                .filter(|&x| 0 == x % 2)
                .for_each(move |x| output_evn_take.borrow_mut().push(x)),
            builder
                .start_tee()
                .filter(|&x| 1 == x % 2)
                .for_each(move |x| output_odd_take.borrow_mut().push(x)),
        );
    builder.add_subgraph(sg);

    let mut hydroflow = builder.build();
    {
        for x in 1..9 {
            ingress_send.give(Some(x));
        }
        ingress_send.flush();

        hydroflow.tick();

        assert_eq!(&[22, 2, 44, 4, 66, 6, 88, 8], &**output_evn.borrow());
        assert_eq!(&[11, 1, 33, 3, 55, 5, 77, 7], &**output_odd.borrow());
    }
}

#[test]
fn test_covid() {
    use crate::scheduled::handoff::VecHandoff;
    use prelude::*;

    type Pid = usize;
    type Name = &'static str;
    type Phone = &'static str;
    type DateTime = usize;

    const TRANSMISSIBLE_DURATION: usize = 14;

    let mut builder = HydroflowBuilder::default();

    let (loop_send, loop_recv) = builder.make_handoff::<VecHandoff<(Pid, DateTime)>, _>();
    let (notifs_send, notifs_recv) = builder.make_handoff::<VecHandoff<(Pid, DateTime)>, _>();

    let (diagnosed_send, diagnosed) =
        builder.add_channel_input::<Option<(Pid, (DateTime, DateTime))>, VecHandoff<_>>();
    let (contacts_send, contacts) =
        builder.add_channel_input::<Option<(Pid, Pid, DateTime)>, VecHandoff<_>>();
    let (peoples_send, peoples) =
        builder.add_channel_input::<Option<(Pid, (Name, Phone))>, VecHandoff<_>>();

    let exposed = loop_recv
        .flat_map(std::convert::identity)
        .map(|(pid, t)| (pid, (t, t + TRANSMISSIBLE_DURATION)))
        .chain(diagnosed.flat_map(std::convert::identity));

    builder.add_subgraph(
        contacts
            .flat_map(std::convert::identity)
            .flat_map(|(pid_a, pid_b, t)| [(pid_a, (pid_b, t)), (pid_b, (pid_a, t))])
            .join(exposed)
            .filter(|(_pid_a, (_pid_b, t_contact), (t_from, t_to))| {
                (t_from..=t_to).contains(&t_contact)
            })
            .map(|(_pid_a, pid_b_t_contact, _t_from_to)| pid_b_t_contact)
            .pivot()
            .map(Some) // For handoff CanReceive.
            .tee(notifs_send, loop_send),
    );

    builder.add_subgraph(
        notifs_recv
            .flat_map(std::convert::identity)
            .join(peoples.flat_map(std::convert::identity))
            .pivot()
            .for_each(|(_pid, exposure_time, (name, phone))| {
                println!(
                    "[{}] To {}: Possible Exposure at t = {}",
                    name, phone, exposure_time
                );
            }),
    );

    let mut hydroflow = builder.build();
    {
        peoples_send.give(Some((101, ("Mingwei S", "+1 650 555 7283"))));
        peoples_send.give(Some((102, ("Justin J", "+1 519 555 3458"))));
        peoples_send.give(Some((103, ("Mae M", "+1 912 555 9129"))));
        peoples_send.flush();

        contacts_send.give(Some((101, 102, 1031))); // Mingwei + Justin
        contacts_send.give(Some((101, 201, 1027))); // Mingwei + Joe
        contacts_send.flush();

        let mae_diag_datetime = 1022;

        diagnosed_send.give(Some((
            103, // Mae
            (
                mae_diag_datetime,
                mae_diag_datetime + TRANSMISSIBLE_DURATION,
            ),
        )));
        diagnosed_send.flush();

        hydroflow.tick();

        contacts_send.give(Some((101, 103, mae_diag_datetime + 6))); // Mingwei + Mae
        contacts_send.flush();

        hydroflow.tick();

        peoples_send.give(Some((103, ("Joe H", "+1 510 555 9999"))));
        peoples_send.flush();

        hydroflow.tick();
    }
}
