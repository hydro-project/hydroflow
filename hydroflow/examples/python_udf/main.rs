use hydroflow_macro::hydroflow_syntax;
use pyo3::{Py, PyAny, PyResult, Python};

#[hydroflow::main]
async fn main() {
    eprintln!("Vec sender starting...");

    let v = vec![1, 2, 3, 4, 5];

    let mut df = hydroflow_syntax! {
        source_iter(v) -> inspect(
            |x| println!("input:\t{:?}", x)
        )
        // Map to tuples
        -> map(|x| (x, 1))
        -> py_udf(r#"
def add(a, b):
    return a + 1
            "#, "add")
        -> map(|x: PyResult<Py<PyAny>>| -> i32 {Python::with_gil(|py| {
            x.unwrap().extract(py).unwrap()
        })})
        -> for_each(|x| println!("output:\t{:?}", x));
    };

    df.run_available();
}
