use hydroflow::hydroflow_syntax;
use hydroflow::scheduled::graph::Hydroflow;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use crate::protocol::{Timestamp, Token};
pub(crate) fn rga_datalog_agg(
    input_recv: UnboundedReceiverStream<(Token, Timestamp)>,
    rga_send: UnboundedSender<(Token, Timestamp)>,
    list_send: UnboundedSender<(Timestamp, Timestamp)>,
) -> Hydroflow<'static> {
    {
        #[allow(unused_qualifications)]
        {
            use ::hydroflow::{var_expr, var_args};
            let mut df = ::hydroflow::scheduled::graph::Hydroflow::new();
            df.__assign_meta_graph(
                "{\"nodes\":[{\"value\":null,\"version\":0},{\"value\":{\"Operator\":\"source_stream(input_recv)\"},\"version\":1},{\"value\":{\"Operator\":\"tee()\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (c, p) : (Token, Timestamp) | (c.ts, p))\"},\"version\":1},{\"value\":{\"Operator\":\"tee()\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (oid, _parent) | oid)\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (_child, parent) | parent)\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (c, p) | (p, c))\"},\"version\":1},{\"value\":{\"Operator\":\"fold_keyed :: < 'static, Timestamp, (Timestamp, Timestamp) >\\n(| |\\n(Timestamp { node_ts : 0, node_id : 0 }, Timestamp\\n{ node_ts : std :: usize :: MAX, node_id : std :: usize :: MAX }), |\\n(first, last) : & mut (Timestamp, Timestamp), s2 : Timestamp |\\n{ if s2 > * first { * first = s2 }; if s2 < * last { * last = s2 }; })\"},\"version\":1},{\"value\":{\"Operator\":\"tee()\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (p, (first, _last)) | (p, first))\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (p, (_first, last)) | (p, last))\"},\"version\":1},{\"value\":{\"Operator\":\"join()\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (_p, (c1, c2)) : (Timestamp, (Timestamp, Timestamp)) | (c1, c2))\"},\"version\":1},{\"value\":{\"Operator\":\"filter(| (s1, s2) | s1 > s2)\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (c, p) | (p, c))\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (c, p) | (p, c))\"},\"version\":1},{\"value\":{\"Operator\":\"fold_keyed :: < 'static, Timestamp, Timestamp >\\n(| | Timestamp { node_ts : 0, node_id : 0 }, | accum : & mut Timestamp, s2 :\\nTimestamp |\\n{\\n    eprintln! (\\\"accum {:?}, s2 {:?}\\\", accum, s2); if s2 > * accum\\n    { * accum = s2 }\\n})\"},\"version\":1},{\"value\":{\"Operator\":\"inspect(| x | println! (\\\"firstLastChild emit {:?}\\\", x))\"},\"version\":1},{\"value\":{\"Operator\":\"union()\"},\"version\":1},{\"value\":{\"Operator\":\"tee()\"},\"version\":1},{\"value\":{\"Operator\":\"filter(| (_s1, s2) | * s2 != Timestamp { node_ts : 0, node_id : 0 })\"},\"version\":1},{\"value\":{\"Operator\":\"join()\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (_parent, (node, next)) | (node, next))\"},\"version\":1},{\"value\":{\"Operator\":\"inspect(| x | println! (\\\"[0]upEdge {:?}\\\", x))\"},\"version\":1},{\"value\":{\"Operator\":\"inspect(| x | println! (\\\"[1]upEdge {:?}\\\", x))\"},\"version\":1},{\"value\":{\"Operator\":\"union()\"},\"version\":1},{\"value\":{\"Operator\":\"difference()\"},\"version\":1},{\"value\":{\"Operator\":\"join()\"},\"version\":1},{\"value\":{\"Operator\":\"map(| (prev, (_, next)) | (prev, next))\"},\"version\":1},{\"value\":{\"Operator\":\"map(| prev | (prev, ()))\"},\"version\":1},{\"value\":{\"Operator\":\"for_each(| (c, p) : (Token, Timestamp) | rga_send.send((c, p)).unwrap())\"},\"version\":1},{\"value\":{\"Operator\":\"for_each(| (first, second) | list_send.send((first, second)).unwrap())\"},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1},{\"value\":{\"Handoff\":{}},\"version\":1}],\"edge_types\":[{\"value\":null,\"version\":0},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":3},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1},{\"value\":\"Value\",\"version\":1}],\"graph\":[{\"value\":null,\"version\":0},{\"value\":[{\"idx\":1,\"version\":1},{\"idx\":2,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":3,\"version\":1},{\"idx\":4,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":2,\"version\":1},{\"idx\":3,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":4,\"version\":1},{\"idx\":5,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":4,\"version\":1},{\"idx\":6,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":8,\"version\":1},{\"idx\":9,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":7,\"version\":1},{\"idx\":33,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":4,\"version\":1},{\"idx\":7,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":9,\"version\":1},{\"idx\":10,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":9,\"version\":1},{\"idx\":11,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":13,\"version\":1},{\"idx\":14,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":12,\"version\":1},{\"idx\":13,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":15,\"version\":1},{\"idx\":12,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":4,\"version\":1},{\"idx\":34,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":16,\"version\":1},{\"idx\":12,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":4,\"version\":1},{\"idx\":35,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":17,\"version\":1},{\"idx\":18,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":14,\"version\":1},{\"idx\":36,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":19,\"version\":1},{\"idx\":20,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":21,\"version\":1},{\"idx\":19,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":18,\"version\":1},{\"idx\":21,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":23,\"version\":1},{\"idx\":19,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":22,\"version\":1},{\"idx\":23,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":24,\"version\":1},{\"idx\":22,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":11,\"version\":1},{\"idx\":37,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":25,\"version\":1},{\"idx\":22,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":20,\"version\":1},{\"idx\":38,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":10,\"version\":1},{\"idx\":39,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":5,\"version\":1},{\"idx\":40,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":6,\"version\":1},{\"idx\":41,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":28,\"version\":1},{\"idx\":29,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":30,\"version\":1},{\"idx\":28,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":27,\"version\":1},{\"idx\":30,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":20,\"version\":1},{\"idx\":42,\"version\":1}],\"version\":3},{\"value\":[{\"idx\":29,\"version\":1},{\"idx\":26,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":2,\"version\":1},{\"idx\":31,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":26,\"version\":1},{\"idx\":32,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":33,\"version\":1},{\"idx\":8,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":34,\"version\":1},{\"idx\":15,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":35,\"version\":1},{\"idx\":16,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":36,\"version\":1},{\"idx\":17,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":37,\"version\":1},{\"idx\":24,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":38,\"version\":1},{\"idx\":25,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":39,\"version\":1},{\"idx\":26,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":40,\"version\":1},{\"idx\":27,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":41,\"version\":1},{\"idx\":27,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":42,\"version\":1},{\"idx\":28,\"version\":1}],\"version\":1}],\"ports\":[{\"value\":null,\"version\":0},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[{\"Path\":\"isListElem\"},\"Elided\"],\"version\":1},{\"value\":[{\"Path\":\"parents\"},\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[{\"Path\":\"firstLastChild\"},\"Elided\"],\"version\":1},{\"value\":[{\"Path\":\"first\"},\"Elided\"],\"version\":1},{\"value\":[{\"Path\":\"last\"},\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",{\"Int\":\"0\"}],\"version\":1},{\"value\":[{\"Path\":\"sib1\"},\"Elided\"],\"version\":3},{\"value\":[\"Elided\",{\"Int\":\"1\"}],\"version\":1},{\"value\":[{\"Path\":\"sib2\"},\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",{\"Int\":\"0\"}],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[\"Elided\",{\"Int\":\"1\"}],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":3},{\"value\":[{\"Path\":\"firstChild\"},\"Elided\"],\"version\":3},{\"value\":[{\"Path\":\"all_nodes\"},\"Elided\"],\"version\":3},{\"value\":[{\"Path\":\"parents\"},\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",{\"Int\":\"0\"}],\"version\":1},{\"value\":[{\"Path\":\"childless\"},\"Elided\"],\"version\":1},{\"value\":[{\"Path\":\"nextSiblingAnc\"},\"Elided\"],\"version\":3},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",\"Elided\"],\"version\":1},{\"value\":[\"Elided\",{\"Path\":\"pos\"}],\"version\":1},{\"value\":[\"Elided\",{\"Path\":\"neg\"}],\"version\":1},{\"value\":[\"Elided\",{\"Int\":\"1\"}],\"version\":1}],\"node_subgraph\":[{\"value\":null,\"version\":0},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":1,\"version\":1},\"version\":1},{\"value\":{\"idx\":1,\"version\":1},\"version\":1},{\"value\":{\"idx\":1,\"version\":1},\"version\":1},{\"value\":{\"idx\":1,\"version\":1},\"version\":1},{\"value\":{\"idx\":2,\"version\":1},\"version\":1},{\"value\":{\"idx\":2,\"version\":1},\"version\":1},{\"value\":{\"idx\":2,\"version\":1},\"version\":1},{\"value\":{\"idx\":2,\"version\":1},\"version\":1},{\"value\":{\"idx\":2,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":3,\"version\":1},\"version\":1},{\"value\":{\"idx\":5,\"version\":1},\"version\":1},{\"value\":{\"idx\":5,\"version\":1},\"version\":1},{\"value\":{\"idx\":5,\"version\":1},\"version\":1},{\"value\":{\"idx\":5,\"version\":1},\"version\":1},{\"value\":{\"idx\":5,\"version\":1},\"version\":1},{\"value\":{\"idx\":4,\"version\":1},\"version\":1},{\"value\":{\"idx\":5,\"version\":1},\"version\":1}],\"subgraph_nodes\":[{\"value\":null,\"version\":0},{\"value\":[{\"idx\":8,\"version\":1},{\"idx\":9,\"version\":1},{\"idx\":10,\"version\":1},{\"idx\":11,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":15,\"version\":1},{\"idx\":16,\"version\":1},{\"idx\":12,\"version\":1},{\"idx\":13,\"version\":1},{\"idx\":14,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":17,\"version\":1},{\"idx\":18,\"version\":1},{\"idx\":21,\"version\":1},{\"idx\":24,\"version\":1},{\"idx\":25,\"version\":1},{\"idx\":22,\"version\":1},{\"idx\":23,\"version\":1},{\"idx\":19,\"version\":1},{\"idx\":20,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":1,\"version\":1},{\"idx\":2,\"version\":1},{\"idx\":3,\"version\":1},{\"idx\":4,\"version\":1},{\"idx\":5,\"version\":1},{\"idx\":6,\"version\":1},{\"idx\":7,\"version\":1},{\"idx\":31,\"version\":1}],\"version\":1},{\"value\":[{\"idx\":27,\"version\":1},{\"idx\":30,\"version\":1},{\"idx\":28,\"version\":1},{\"idx\":29,\"version\":1},{\"idx\":26,\"version\":1},{\"idx\":32,\"version\":1}],\"version\":1}],\"subgraph_stratum\":[{\"value\":null,\"version\":0},{\"value\":1,\"version\":1},{\"value\":0,\"version\":1},{\"value\":1,\"version\":1},{\"value\":0,\"version\":1},{\"value\":1,\"version\":1}],\"node_varnames\":[{\"value\":null,\"version\":0},{\"value\":\"edges\",\"version\":1},{\"value\":\"edges\",\"version\":1},{\"value\":\"insertAfter\",\"version\":1},{\"value\":\"insertAfter\",\"version\":1},{\"value\":\"isListElem\",\"version\":1},{\"value\":\"parents\",\"version\":1},{\"value\":\"firstLastChild\",\"version\":1},{\"value\":\"firstLastChild\",\"version\":1},{\"value\":\"firstLastChild\",\"version\":1},{\"value\":\"firstChild\",\"version\":1},{\"value\":\"lastChild\",\"version\":1},{\"value\":\"sibling\",\"version\":1},{\"value\":\"sibling\",\"version\":1},{\"value\":\"sibling\",\"version\":1},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":\"nextSibling\",\"version\":1},{\"value\":\"nextSibling\",\"version\":1},{\"value\":\"nextSiblingAnc\",\"version\":1},{\"value\":\"nextSiblingAnc\",\"version\":1},{\"value\":null,\"version\":0},{\"value\":\"upEdge\",\"version\":1},{\"value\":\"upEdge\",\"version\":1},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":\"nextElem\",\"version\":1},{\"value\":\"childless\",\"version\":1},{\"value\":\"ne_join\",\"version\":1},{\"value\":\"ne_join\",\"version\":1}],\"flow_props\":[{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":14,\"lattice_flow_type\":null},\"version\":1},{\"value\":{\"star_ord\":14,\"lattice_flow_type\":null},\"version\":1},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":14,\"lattice_flow_type\":null},\"version\":3},{\"value\":{\"star_ord\":19,\"lattice_flow_type\":null},\"version\":1},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":24,\"lattice_flow_type\":null},\"version\":1},{\"value\":{\"star_ord\":24,\"lattice_flow_type\":null},\"version\":1},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":19,\"lattice_flow_type\":null},\"version\":1},{\"value\":{\"star_ord\":19,\"lattice_flow_type\":null},\"version\":3},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":32,\"lattice_flow_type\":null},\"version\":1},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":19,\"lattice_flow_type\":null},\"version\":3},{\"value\":{\"star_ord\":32,\"lattice_flow_type\":null},\"version\":1},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":34,\"lattice_flow_type\":null},\"version\":1},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":14,\"lattice_flow_type\":null},\"version\":1},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":19,\"lattice_flow_type\":null},\"version\":1},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":null,\"version\":0},{\"value\":{\"star_ord\":19,\"lattice_flow_type\":null},\"version\":1}],\"subgraph_laziness\":[{\"value\":null,\"version\":0}]}",
            );
            df.__assign_diagnostics("[]");
            let (hoff_33v1_send, hoff_33v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(33v1)");
            let (hoff_34v1_send, hoff_34v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(34v1)");
            let (hoff_35v1_send, hoff_35v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(35v1)");
            let (hoff_36v1_send, hoff_36v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(36v1)");
            let (hoff_37v1_send, hoff_37v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(37v1)");
            let (hoff_38v1_send, hoff_38v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(38v1)");
            let (hoff_39v1_send, hoff_39v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(39v1)");
            let (hoff_40v1_send, hoff_40v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(40v1)");
            let (hoff_41v1_send, hoff_41v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(41v1)");
            let (hoff_42v1_send, hoff_42v1_recv) = df
                .make_edge::<
                    _,
                    ::hydroflow::scheduled::handoff::VecHandoff<_>,
                >("handoff GraphNodeId(42v1)");
            let mut sg_4v1_node_1v1_stream = {
                #[inline(always)]
                fn check_stream<
                    Stream: ::hydroflow::futures::stream::Stream<Item = Item>
                        + ::std::marker::Unpin,
                    Item,
                >(
                    stream: Stream,
                ) -> impl ::hydroflow::futures::stream::Stream<
                    Item = Item,
                > + ::std::marker::Unpin {
                    stream
                }
                check_stream(input_recv)
            };
            let sg_1v1_node_8v1_groupbydata = df
                .add_state(
                    ::std::cell::RefCell::new(
                        ::hydroflow::rustc_hash::FxHashMap::<
                            Timestamp,
                            (Timestamp, Timestamp),
                        >::default(),
                    ),
                );
            let sg_2v1_node_12v1_joindata_lhs = df
                .add_state(
                    std::cell::RefCell::new(
                        ::hydroflow::util::monotonic_map::MonotonicMap::new_init(
                            ::hydroflow::compiled::pull::HalfSetJoinState::default(),
                        ),
                    ),
                );
            let sg_2v1_node_12v1_joindata_rhs = df
                .add_state(
                    std::cell::RefCell::new(
                        ::hydroflow::util::monotonic_map::MonotonicMap::new_init(
                            ::hydroflow::compiled::pull::HalfSetJoinState::default(),
                        ),
                    ),
                );
            let sg_3v1_node_17v1_groupbydata = df
                .add_state(
                    ::std::cell::RefCell::new(
                        ::hydroflow::rustc_hash::FxHashMap::<
                            Timestamp,
                            Timestamp,
                        >::default(),
                    ),
                );
            let sg_3v1_node_22v1_joindata_lhs = df
                .add_state(
                    std::cell::RefCell::new(
                        ::hydroflow::util::monotonic_map::MonotonicMap::new_init(
                            ::hydroflow::compiled::pull::HalfSetJoinState::default(),
                        ),
                    ),
                );
            let sg_3v1_node_22v1_joindata_rhs = df
                .add_state(
                    std::cell::RefCell::new(
                        ::hydroflow::util::monotonic_map::MonotonicMap::new_init(
                            ::hydroflow::compiled::pull::HalfSetJoinState::default(),
                        ),
                    ),
                );
            let sg_5v1_node_27v1_antijoindata_neg = df
                .add_state(
                    std::cell::RefCell::new(
                        ::hydroflow::util::monotonic_map::MonotonicMap::<
                            _,
                            ::hydroflow::rustc_hash::FxHashSet<_>,
                        >::default(),
                    ),
                );
            let sg_5v1_node_27v1_antijoindata_pos = df
                .add_state(
                    std::cell::RefCell::new(
                        ::hydroflow::util::monotonic_map::MonotonicMap::<
                            _,
                            ::hydroflow::rustc_hash::FxHashSet<_>,
                        >::default(),
                    ),
                );
            let sg_5v1_node_28v1_joindata_lhs = df
                .add_state(
                    std::cell::RefCell::new(
                        ::hydroflow::util::monotonic_map::MonotonicMap::new_init(
                            ::hydroflow::compiled::pull::HalfSetJoinState::default(),
                        ),
                    ),
                );
            let sg_5v1_node_28v1_joindata_rhs = df
                .add_state(
                    std::cell::RefCell::new(
                        ::hydroflow::util::monotonic_map::MonotonicMap::new_init(
                            ::hydroflow::compiled::pull::HalfSetJoinState::default(),
                        ),
                    ),
                );
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(4v1)",
                0,
                (),
                (
                    hoff_33v1_send,
                    (
                        hoff_34v1_send,
                        (hoff_35v1_send, (hoff_40v1_send, (hoff_41v1_send, ()))),
                    ),
                ),
                false,
                move |
                    context,
                    (),
                    (
                        hoff_33v1_send,
                        (
                            hoff_34v1_send,
                            (hoff_35v1_send, (hoff_40v1_send, (hoff_41v1_send, ()))),
                        ),
                    )|
                {
                    let hoff_33v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_33v1_send.give(Some(v));
                    });
                    let hoff_34v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_34v1_send.give(Some(v));
                    });
                    let hoff_35v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_35v1_send.give(Some(v));
                    });
                    let hoff_40v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_40v1_send.give(Some(v));
                    });
                    let hoff_41v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_41v1_send.give(Some(v));
                    });
                    let op_1v1 = std::iter::from_fn(|| {
                        match ::hydroflow::futures::stream::Stream::poll_next(
                            ::std::pin::Pin::new(&mut sg_4v1_node_1v1_stream),
                            &mut std::task::Context::from_waker(&context.waker()),
                        ) {
                            std::task::Poll::Ready(maybe) => maybe,
                            std::task::Poll::Pending => None,
                        }
                    });
                    let op_1v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_1v1__source_stream__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_1v1__source_stream__loc_unknown_start_0_0_end_0_0(op_1v1)
                    };
                    let op_31v1 = ::hydroflow::pusherator::for_each::ForEach::new(|
                        (c, p): (Token, Timestamp)|
                    rga_send.send((c, p)).unwrap());
                    let op_31v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_31v1__for_each__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_31v1__for_each__loc_unknown_start_0_0_end_0_0(op_31v1)
                    };
                    let op_7v1 = ::hydroflow::pusherator::map::Map::new(
                        |(c, p)| (p, c),
                        hoff_33v1_send,
                    );
                    let op_7v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_7v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_7v1__map__loc_unknown_start_0_0_end_0_0(op_7v1)
                    };
                    let op_6v1 = ::hydroflow::pusherator::map::Map::new(
                        |(_child, parent)| parent,
                        hoff_41v1_send,
                    );
                    let op_6v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_6v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_6v1__map__loc_unknown_start_0_0_end_0_0(op_6v1)
                    };
                    let op_5v1 = ::hydroflow::pusherator::map::Map::new(
                        |(oid, _parent)| oid,
                        hoff_40v1_send,
                    );
                    let op_5v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_5v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_5v1__map__loc_unknown_start_0_0_end_0_0(op_5v1)
                    };
                    let op_4v1 = ::hydroflow::pusherator::tee::Tee::new(
                        op_7v1,
                        ::hydroflow::pusherator::tee::Tee::new(
                            op_5v1,
                            ::hydroflow::pusherator::tee::Tee::new(
                                op_6v1,
                                ::hydroflow::pusherator::tee::Tee::new(
                                    hoff_34v1_send,
                                    hoff_35v1_send,
                                ),
                            ),
                        ),
                    );
                    let op_4v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_4v1__tee__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_4v1__tee__loc_unknown_start_0_0_end_0_0(op_4v1)
                    };
                    let op_3v1 = ::hydroflow::pusherator::map::Map::new(
                        |(c, p): (Token, Timestamp)| (c.ts, p),
                        op_4v1,
                    );
                    let op_3v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_3v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_3v1__map__loc_unknown_start_0_0_end_0_0(op_3v1)
                    };
                    let op_2v1 = ::hydroflow::pusherator::tee::Tee::new(
                        op_3v1,
                        op_31v1,
                    );
                    let op_2v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_2v1__tee__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_2v1__tee__loc_unknown_start_0_0_end_0_0(op_2v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_1v1, op_2v1);
                },
            );
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(1v1)",
                1,
                (hoff_33v1_recv, ()),
                (hoff_37v1_send, (hoff_39v1_send, ())),
                false,
                move |
                    context,
                    (hoff_33v1_recv, ()),
                    (hoff_37v1_send, (hoff_39v1_send, ()))|
                {
                    let mut hoff_33v1_recv = hoff_33v1_recv.borrow_mut_swap();
                    let hoff_33v1_recv = hoff_33v1_recv.drain(..);
                    let hoff_37v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_37v1_send.give(Some(v));
                    });
                    let hoff_39v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_39v1_send.give(Some(v));
                    });
                    let mut sg_1v1_node_8v1_hashtable = context
                        .state_ref(sg_1v1_node_8v1_groupbydata)
                        .borrow_mut();
                    let op_8v1 = {
                        let replay_iter = if context.is_first_run_this_tick() {
                            ::hydroflow::itertools::Either::Left(
                                sg_1v1_node_8v1_hashtable.iter(),
                            )
                        } else {
                            ::hydroflow::itertools::Either::Right(::std::iter::empty())
                        };
                        let new_items_iter = {
                            #[inline(always)]
                            fn check_input<Iter, A, B>(
                                iter: Iter,
                            ) -> impl ::std::iter::Iterator<Item = (A, B)>
                            where
                                Iter: std::iter::Iterator<Item = (A, B)>,
                                A: ::std::clone::Clone,
                                B: ::std::clone::Clone,
                            {
                                iter
                            }
                            check_input(hoff_33v1_recv)
                                .map(|(key, val)| {
                                    /// A: accumulator type
                                    /// T: iterator item type
                                    /// O: output type
                                    #[inline(always)]
                                    fn call_comb_type<A, T, O>(
                                        a: &mut A,
                                        t: T,
                                        f: impl Fn(&mut A, T) -> O,
                                    ) -> O {
                                        (f)(a, t)
                                    }
                                    #[allow(unknown_lints, clippy::unwrap_or_default)]
                                    let entry = sg_1v1_node_8v1_hashtable
                                        .entry(key.clone())
                                        .or_insert_with(|| (
                                            Timestamp {
                                                node_ts: 0,
                                                node_id: 0,
                                            },
                                            Timestamp {
                                                node_ts: std::usize::MAX,
                                                node_id: std::usize::MAX,
                                            },
                                        ));
                                    #[allow(clippy::redundant_closure_call)]
                                    call_comb_type(
                                        entry,
                                        val,
                                        |(first, last): &mut (Timestamp, Timestamp), s2: Timestamp| {
                                            if s2 > *first {
                                                *first = s2;
                                            }
                                            if s2 < *last {
                                                *last = s2;
                                            }
                                        },
                                    );
                                    (key, entry.clone())
                                })
                        };
                        replay_iter
                            .map(
                                #[allow(
                                    unknown_lints,
                                    suspicious_double_ref_op,
                                    clippy::clone_on_copy
                                )]
                                |(k, v)| (k.clone(), v.clone()),
                            )
                            .chain(new_items_iter)
                    };
                    let op_8v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_8v1__fold_keyed__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_8v1__fold_keyed__loc_unknown_start_0_0_end_0_0(op_8v1)
                    };
                    let op_11v1 = ::hydroflow::pusherator::map::Map::new(
                        |(p, (_first, last))| (p, last),
                        hoff_37v1_send,
                    );
                    let op_11v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_11v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_11v1__map__loc_unknown_start_0_0_end_0_0(op_11v1)
                    };
                    let op_10v1 = ::hydroflow::pusherator::map::Map::new(
                        |(p, (first, _last))| (p, first),
                        hoff_39v1_send,
                    );
                    let op_10v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_10v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_10v1__map__loc_unknown_start_0_0_end_0_0(op_10v1)
                    };
                    let op_9v1 = ::hydroflow::pusherator::tee::Tee::new(
                        op_10v1,
                        op_11v1,
                    );
                    let op_9v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_9v1__tee__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_9v1__tee__loc_unknown_start_0_0_end_0_0(op_9v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_8v1, op_9v1);
                    context.schedule_subgraph(context.current_subgraph(), false);
                },
            );
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(2v1)",
                0,
                (hoff_34v1_recv, (hoff_35v1_recv, ())),
                (hoff_36v1_send, ()),
                false,
                move |
                    context,
                    (hoff_34v1_recv, (hoff_35v1_recv, ())),
                    (hoff_36v1_send, ())|
                {
                    let mut hoff_34v1_recv = hoff_34v1_recv.borrow_mut_swap();
                    let hoff_34v1_recv = hoff_34v1_recv.drain(..);
                    let mut hoff_35v1_recv = hoff_35v1_recv.borrow_mut_swap();
                    let hoff_35v1_recv = hoff_35v1_recv.drain(..);
                    let hoff_36v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_36v1_send.give(Some(v));
                    });
                    let op_15v1 = hoff_34v1_recv.map(|(c, p)| (p, c));
                    let op_15v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_15v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_15v1__map__loc_unknown_start_0_0_end_0_0(op_15v1)
                    };
                    let op_16v1 = hoff_35v1_recv.map(|(c, p)| (p, c));
                    let op_16v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_16v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_16v1__map__loc_unknown_start_0_0_end_0_0(op_16v1)
                    };
                    let mut sg_2v1_node_12v1_joindata_lhs_borrow = context
                        .state_ref(sg_2v1_node_12v1_joindata_lhs)
                        .borrow_mut();
                    let mut sg_2v1_node_12v1_joindata_rhs_borrow = context
                        .state_ref(sg_2v1_node_12v1_joindata_rhs)
                        .borrow_mut();
                    let op_12v1 = {
                        #[inline(always)]
                        fn check_inputs<'a, K, I1, V1, I2, V2>(
                            lhs: I1,
                            rhs: I2,
                            lhs_state: &'a mut ::hydroflow::compiled::pull::HalfSetJoinState<
                                K,
                                V1,
                                V2,
                            >,
                            rhs_state: &'a mut ::hydroflow::compiled::pull::HalfSetJoinState<
                                K,
                                V2,
                                V1,
                            >,
                            is_new_tick: bool,
                        ) -> impl 'a + Iterator<Item = (K, (V1, V2))>
                        where
                            K: Eq + std::hash::Hash + Clone,
                            V1: Clone + ::std::cmp::Eq,
                            V2: Clone + ::std::cmp::Eq,
                            I1: 'a + Iterator<Item = (K, V1)>,
                            I2: 'a + Iterator<Item = (K, V2)>,
                        {
                            ::hydroflow::compiled::pull::symmetric_hash_join_into_iter(
                                lhs,
                                rhs,
                                lhs_state,
                                rhs_state,
                                is_new_tick,
                            )
                        }
                        check_inputs(
                            op_15v1,
                            op_16v1,
                            &mut *sg_2v1_node_12v1_joindata_lhs_borrow
                                .get_mut_clear(context.current_tick()),
                            &mut *sg_2v1_node_12v1_joindata_rhs_borrow
                                .get_mut_clear(context.current_tick()),
                            context.is_first_run_this_tick(),
                        )
                    };
                    let op_12v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_12v1__join__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_12v1__join__loc_unknown_start_0_0_end_0_0(op_12v1)
                    };
                    let op_13v1 = op_12v1
                        .map(|(_p, (c1, c2)): (Timestamp, (Timestamp, Timestamp))| (
                            c1,
                            c2,
                        ));
                    let op_13v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_13v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_13v1__map__loc_unknown_start_0_0_end_0_0(op_13v1)
                    };
                    let op_14v1 = op_13v1.filter(|(s1, s2)| s1 > s2);
                    let op_14v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_14v1__filter__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_14v1__filter__loc_unknown_start_0_0_end_0_0(op_14v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_14v1, hoff_36v1_send);
                },
            );
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(3v1)",
                1,
                (hoff_36v1_recv, (hoff_37v1_recv, (hoff_38v1_recv, ()))),
                (hoff_38v1_send, (hoff_42v1_send, ())),
                false,
                move |
                    context,
                    (hoff_36v1_recv, (hoff_37v1_recv, (hoff_38v1_recv, ()))),
                    (hoff_38v1_send, (hoff_42v1_send, ()))|
                {
                    let mut hoff_36v1_recv = hoff_36v1_recv.borrow_mut_swap();
                    let hoff_36v1_recv = hoff_36v1_recv.drain(..);
                    let mut hoff_37v1_recv = hoff_37v1_recv.borrow_mut_swap();
                    let hoff_37v1_recv = hoff_37v1_recv.drain(..);
                    let mut hoff_38v1_recv = hoff_38v1_recv.borrow_mut_swap();
                    let hoff_38v1_recv = hoff_38v1_recv.drain(..);
                    let hoff_38v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_38v1_send.give(Some(v));
                    });
                    let hoff_42v1_send = ::hydroflow::pusherator::for_each::ForEach::new(|
                        v|
                    {
                        hoff_42v1_send.give(Some(v));
                    });
                    let mut sg_3v1_node_17v1_hashtable = context
                        .state_ref(sg_3v1_node_17v1_groupbydata)
                        .borrow_mut();
                    let op_17v1 = {
                        let replay_iter = if context.is_first_run_this_tick() {
                            ::hydroflow::itertools::Either::Left(
                                sg_3v1_node_17v1_hashtable.iter(),
                            )
                        } else {
                            ::hydroflow::itertools::Either::Right(::std::iter::empty())
                        };
                        let new_items_iter = {
                            #[inline(always)]
                            fn check_input<Iter, A, B>(
                                iter: Iter,
                            ) -> impl ::std::iter::Iterator<Item = (A, B)>
                            where
                                Iter: std::iter::Iterator<Item = (A, B)>,
                                A: ::std::clone::Clone,
                                B: ::std::clone::Clone,
                            {
                                iter
                            }
                            check_input(hoff_36v1_recv)
                                .map(|(key, val)| {
                                    /// A: accumulator type
                                    /// T: iterator item type
                                    /// O: output type
                                    #[inline(always)]
                                    fn call_comb_type<A, T, O>(
                                        a: &mut A,
                                        t: T,
                                        f: impl Fn(&mut A, T) -> O,
                                    ) -> O {
                                        (f)(a, t)
                                    }
                                    #[allow(unknown_lints, clippy::unwrap_or_default)]
                                    let entry = sg_3v1_node_17v1_hashtable
                                        .entry(key.clone())
                                        .or_insert_with(|| Timestamp {
                                            node_ts: 0,
                                            node_id: 0,
                                        });
                                    #[allow(clippy::redundant_closure_call)]
                                    call_comb_type(
                                        entry,
                                        val,
                                        |accum: &mut Timestamp, s2: Timestamp| {
                                            {
                                                eprintln!("{}", 
                                                    format_args!("accum {0:?}, s2 {1:?}\n", accum, s2),
                                                );
                                            };
                                            if s2 > *accum {
                                                *accum = s2;
                                            }
                                        },
                                    );
                                    (key, entry.clone())
                                })
                        };
                        replay_iter
                            .map(
                                #[allow(
                                    unknown_lints,
                                    suspicious_double_ref_op,
                                    clippy::clone_on_copy
                                )]
                                |(k, v)| (k.clone(), v.clone()),
                            )
                            .chain(new_items_iter)
                    };
                    let op_17v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_17v1__fold_keyed__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_17v1__fold_keyed__loc_unknown_start_0_0_end_0_0(op_17v1)
                    };
                    let op_18v1 = op_17v1
                        .inspect(|x| {
                            println!("{}", 
                                format_args!("firstLastChild emit {0:?}\n", x),
                            );
                        });
                    let op_18v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_18v1__inspect__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_18v1__inspect__loc_unknown_start_0_0_end_0_0(op_18v1)
                    };
                    let op_21v1 = op_18v1
                        .filter(|(_s1, s2)| {
                            *s2
                                != Timestamp {
                                    node_ts: 0,
                                    node_id: 0,
                                }
                        });
                    let op_21v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_21v1__filter__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_21v1__filter__loc_unknown_start_0_0_end_0_0(op_21v1)
                    };
                    let op_24v1 = hoff_37v1_recv
                        .inspect(|x| {
                            println!("{}", format_args!("[0]upEdge {0:?}\n", x));
                        });
                    let op_24v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_24v1__inspect__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_24v1__inspect__loc_unknown_start_0_0_end_0_0(op_24v1)
                    };
                    let op_25v1 = hoff_38v1_recv
                        .inspect(|x| {
                            println!("{}", format_args!("[1]upEdge {0:?}\n", x));
                        });
                    let op_25v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_25v1__inspect__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_25v1__inspect__loc_unknown_start_0_0_end_0_0(op_25v1)
                    };
                    let mut sg_3v1_node_22v1_joindata_lhs_borrow = context
                        .state_ref(sg_3v1_node_22v1_joindata_lhs)
                        .borrow_mut();
                    let mut sg_3v1_node_22v1_joindata_rhs_borrow = context
                        .state_ref(sg_3v1_node_22v1_joindata_rhs)
                        .borrow_mut();
                    let op_22v1 = {
                        #[inline(always)]
                        fn check_inputs<'a, K, I1, V1, I2, V2>(
                            lhs: I1,
                            rhs: I2,
                            lhs_state: &'a mut ::hydroflow::compiled::pull::HalfSetJoinState<
                                K,
                                V1,
                                V2,
                            >,
                            rhs_state: &'a mut ::hydroflow::compiled::pull::HalfSetJoinState<
                                K,
                                V2,
                                V1,
                            >,
                            is_new_tick: bool,
                        ) -> impl 'a + Iterator<Item = (K, (V1, V2))>
                        where
                            K: Eq + std::hash::Hash + Clone,
                            V1: Clone + ::std::cmp::Eq,
                            V2: Clone + ::std::cmp::Eq,
                            I1: 'a + Iterator<Item = (K, V1)>,
                            I2: 'a + Iterator<Item = (K, V2)>,
                        {
                            ::hydroflow::compiled::pull::symmetric_hash_join_into_iter(
                                lhs,
                                rhs,
                                lhs_state,
                                rhs_state,
                                is_new_tick,
                            )
                        }
                        check_inputs(
                            op_24v1,
                            op_25v1,
                            &mut *sg_3v1_node_22v1_joindata_lhs_borrow
                                .get_mut_clear(context.current_tick()),
                            &mut *sg_3v1_node_22v1_joindata_rhs_borrow
                                .get_mut_clear(context.current_tick()),
                            context.is_first_run_this_tick(),
                        )
                    };
                    let op_22v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_22v1__join__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_22v1__join__loc_unknown_start_0_0_end_0_0(op_22v1)
                    };
                    let op_23v1 = op_22v1
                        .map(|(_parent, (node, next))| (node, next));
                    let op_23v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_23v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_23v1__map__loc_unknown_start_0_0_end_0_0(op_23v1)
                    };
                    let op_19v1 = {
                        #[allow(unused)]
                        #[inline(always)]
                        fn check_inputs<
                            A: ::std::iter::Iterator<Item = Item>,
                            B: ::std::iter::Iterator<Item = Item>,
                            Item,
                        >(a: A, b: B) -> impl ::std::iter::Iterator<Item = Item> {
                            a.chain(b)
                        }
                        check_inputs(op_21v1, op_23v1)
                    };
                    let op_19v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_19v1__union__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_19v1__union__loc_unknown_start_0_0_end_0_0(op_19v1)
                    };
                    let op_20v1 = ::hydroflow::pusherator::tee::Tee::new(
                        hoff_42v1_send,
                        hoff_38v1_send,
                    );
                    let op_20v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_20v1__tee__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_20v1__tee__loc_unknown_start_0_0_end_0_0(op_20v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_19v1, op_20v1);
                    context.schedule_subgraph(context.current_subgraph(), false);
                },
            );
            df.add_subgraph_stratified(
                "Subgraph GraphSubgraphId(5v1)",
                1,
                (
                    hoff_39v1_recv,
                    (hoff_40v1_recv, (hoff_41v1_recv, (hoff_42v1_recv, ()))),
                ),
                (),
                false,
                move |
                    context,
                    (
                        hoff_39v1_recv,
                        (hoff_40v1_recv, (hoff_41v1_recv, (hoff_42v1_recv, ()))),
                    ),
                    ()|
                {
                    let mut hoff_39v1_recv = hoff_39v1_recv.borrow_mut_swap();
                    let hoff_39v1_recv = hoff_39v1_recv.drain(..);
                    let mut hoff_40v1_recv = hoff_40v1_recv.borrow_mut_swap();
                    let hoff_40v1_recv = hoff_40v1_recv.drain(..);
                    let mut hoff_41v1_recv = hoff_41v1_recv.borrow_mut_swap();
                    let hoff_41v1_recv = hoff_41v1_recv.drain(..);
                    let mut hoff_42v1_recv = hoff_42v1_recv.borrow_mut_swap();
                    let hoff_42v1_recv = hoff_42v1_recv.drain(..);
                    let hoff_40v1_recv = hoff_40v1_recv.map(|k| (k, ()));
                    let mut sg_5v1_node_27v1_antijoindata_neg_borrow = context
                        .state_ref(sg_5v1_node_27v1_antijoindata_neg)
                        .borrow_mut();
                    let mut sg_5v1_node_27v1_antijoindata_pos_borrow = context
                        .state_ref(sg_5v1_node_27v1_antijoindata_pos)
                        .borrow_mut();
                    let op_27v1 = {
                        /// Limit error propagation by bounding locally, erasing output iterator type.
                        #[inline(always)]
                        fn check_inputs<'a, K, I1, V, I2>(
                            input_neg: I1,
                            input_pos: I2,
                            neg_state: &'a mut ::hydroflow::rustc_hash::FxHashSet<K>,
                            pos_state: &'a mut ::hydroflow::rustc_hash::FxHashSet<
                                (K, V),
                            >,
                            is_new_tick: bool,
                        ) -> impl 'a + Iterator<Item = (K, V)>
                        where
                            K: Eq + ::std::hash::Hash + Clone,
                            V: Eq + ::std::hash::Hash + Clone,
                            I1: 'a + Iterator<Item = K>,
                            I2: 'a + Iterator<Item = (K, V)>,
                        {
                            neg_state.extend(input_neg);
                            ::hydroflow::compiled::pull::anti_join_into_iter(
                                input_pos,
                                neg_state,
                                pos_state,
                                is_new_tick,
                            )
                        }
                        check_inputs(
                            hoff_41v1_recv,
                            hoff_40v1_recv,
                            &mut *sg_5v1_node_27v1_antijoindata_neg_borrow
                                .get_mut_clear(context.current_tick()),
                            &mut *sg_5v1_node_27v1_antijoindata_pos_borrow
                                .get_mut_clear(context.current_tick()),
                            context.is_first_run_this_tick(),
                        )
                    };
                    let op_27v1 = op_27v1.map(|(k, ())| k);
                    let op_27v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_27v1__difference__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_27v1__difference__loc_unknown_start_0_0_end_0_0(op_27v1)
                    };
                    let op_30v1 = op_27v1.map(|prev| (prev, ()));
                    let op_30v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_30v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_30v1__map__loc_unknown_start_0_0_end_0_0(op_30v1)
                    };
                    let mut sg_5v1_node_28v1_joindata_lhs_borrow = context
                        .state_ref(sg_5v1_node_28v1_joindata_lhs)
                        .borrow_mut();
                    let mut sg_5v1_node_28v1_joindata_rhs_borrow = context
                        .state_ref(sg_5v1_node_28v1_joindata_rhs)
                        .borrow_mut();
                    let op_28v1 = {
                        #[inline(always)]
                        fn check_inputs<'a, K, I1, V1, I2, V2>(
                            lhs: I1,
                            rhs: I2,
                            lhs_state: &'a mut ::hydroflow::compiled::pull::HalfSetJoinState<
                                K,
                                V1,
                                V2,
                            >,
                            rhs_state: &'a mut ::hydroflow::compiled::pull::HalfSetJoinState<
                                K,
                                V2,
                                V1,
                            >,
                            is_new_tick: bool,
                        ) -> impl 'a + Iterator<Item = (K, (V1, V2))>
                        where
                            K: Eq + std::hash::Hash + Clone,
                            V1: Clone + ::std::cmp::Eq,
                            V2: Clone + ::std::cmp::Eq,
                            I1: 'a + Iterator<Item = (K, V1)>,
                            I2: 'a + Iterator<Item = (K, V2)>,
                        {
                            ::hydroflow::compiled::pull::symmetric_hash_join_into_iter(
                                lhs,
                                rhs,
                                lhs_state,
                                rhs_state,
                                is_new_tick,
                            )
                        }
                        check_inputs(
                            op_30v1,
                            hoff_42v1_recv,
                            &mut *sg_5v1_node_28v1_joindata_lhs_borrow
                                .get_mut_clear(context.current_tick()),
                            &mut *sg_5v1_node_28v1_joindata_rhs_borrow
                                .get_mut_clear(context.current_tick()),
                            context.is_first_run_this_tick(),
                        )
                    };
                    let op_28v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_28v1__join__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_28v1__join__loc_unknown_start_0_0_end_0_0(op_28v1)
                    };
                    let op_29v1 = op_28v1.map(|(prev, (_, next))| (prev, next));
                    let op_29v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_29v1__map__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_29v1__map__loc_unknown_start_0_0_end_0_0(op_29v1)
                    };
                    let op_26v1 = {
                        #[allow(unused)]
                        #[inline(always)]
                        fn check_inputs<
                            A: ::std::iter::Iterator<Item = Item>,
                            B: ::std::iter::Iterator<Item = Item>,
                            Item,
                        >(a: A, b: B) -> impl ::std::iter::Iterator<Item = Item> {
                            a.chain(b)
                        }
                        check_inputs(op_29v1, hoff_39v1_recv)
                    };
                    let op_26v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_26v1__union__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::std::iter::Iterator<Item = Item>,
                        >(input: Input) -> impl ::std::iter::Iterator<Item = Item> {
                            struct Pull<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::std::iter::Iterator<Item = Item>,
                            > Iterator for Pull<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn next(&mut self) -> Option<Self::Item> {
                                    self.inner.next()
                                }
                                #[inline(always)]
                                fn size_hint(&self) -> (usize, Option<usize>) {
                                    self.inner.size_hint()
                                }
                            }
                            Pull { inner: input }
                        }
                        op_26v1__union__loc_unknown_start_0_0_end_0_0(op_26v1)
                    };
                    let op_32v1 = ::hydroflow::pusherator::for_each::ForEach::new(|
                        (first, second)|
                    list_send.send((first, second)).unwrap());
                    let op_32v1 = {
                        #[allow(non_snake_case)]
                        #[inline(always)]
                        pub fn op_32v1__for_each__loc_unknown_start_0_0_end_0_0<
                            Item,
                            Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        >(
                            input: Input,
                        ) -> impl ::hydroflow::pusherator::Pusherator<Item = Item> {
                            struct Push<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > {
                                inner: Input,
                            }
                            impl<
                                Item,
                                Input: ::hydroflow::pusherator::Pusherator<Item = Item>,
                            > ::hydroflow::pusherator::Pusherator
                            for Push<Item, Input> {
                                type Item = Item;
                                #[inline(always)]
                                fn give(&mut self, item: Self::Item) {
                                    self.inner.give(item)
                                }
                            }
                            Push { inner: input }
                        }
                        op_32v1__for_each__loc_unknown_start_0_0_end_0_0(op_32v1)
                    };
                    #[inline(always)]
                    fn check_pivot_run<
                        Pull: ::std::iter::Iterator<Item = Item>,
                        Push: ::hydroflow::pusherator::Pusherator<Item = Item>,
                        Item,
                    >(pull: Pull, push: Push) {
                        ::hydroflow::pusherator::pivot::Pivot::new(pull, push).run();
                    }
                    check_pivot_run(op_26v1, op_32v1);
                },
            );
            df
        }
    }
}
