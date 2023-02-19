use hydroflow::hydroflow_syntax;

fn main() {
    let mut df = hydroflow_syntax! {
        a = b;
        b = c;
        c = d;
        d = e;
        e = f;
        f = g;
        g = h;
        h = i;
        i = j;
        j = a;

        j -> null();
    };
    df.run_available();
}
