use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        hydro_lab() -> for_each(std::mem::drop);
    };
    df.run_available();
}
