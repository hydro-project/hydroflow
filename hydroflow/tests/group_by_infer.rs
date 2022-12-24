pub struct SubordResponse {
    pub xid: &'static str,
    pub mtype: u32,
}

#[test]
pub fn test_basic() {
    let mut df = hydroflow::hydroflow_syntax! {
        source_iter([
            SubordResponse { xid: "123", mtype: 33 },
            SubordResponse { xid: "123", mtype: 52 },
            SubordResponse { xid: "123", mtype: 72 },
            SubordResponse { xid: "123", mtype: 83 },
            SubordResponse { xid: "123", mtype: 78 },
        ])
            -> map(|m: SubordResponse| (m.xid, 1))
            -> group_by::<'static>(|| 0, |old: &mut u32, val: u32| *old += val)
            -> for_each(|(k, v)| println!("{}: {}", k, v));
    };
    df.run_available();
}
