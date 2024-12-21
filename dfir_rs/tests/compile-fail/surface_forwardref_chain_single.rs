use dfir_rs::dfir_syntax;

fn main() {
    let mut df = dfir_syntax! {
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
