use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        id = identity();
        inn = [0]id;
        out = id[0];
        out[0] -> [0]inn; // Error, equivalent to: `[0][0]id[0][0]`
    };
    df.run_available();
}
