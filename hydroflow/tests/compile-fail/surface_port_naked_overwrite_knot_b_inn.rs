use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        pivot = merge() -> tee();

        x_0 = [0]pivot;
        x_1 = [1]pivot;

        x_0 -> [0]x_0;
        x_1[0] -> [1]x_1; // Error: `[1][1]pivot`
    };
    df.run_available();
}
