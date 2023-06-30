#![cfg(feature = "python")]

use hydroflow::{assert_graphvis_snapshots, hydroflow_syntax};
use multiplatform_test::multiplatform_test;
use pyo3::prelude::*;

#[multiplatform_test(test)]
pub fn test_python_basic() {
    let mut hf = hydroflow_syntax! {
        source_iter(0..10)
            -> map(|x| (x,))
            -> py_udf(r#"
def fib(n):
    if n < 2:
        return n
    else:
        return fib(n - 2) + fib(n - 1)
            "#, "fib")
            -> map(|x: PyResult<Py<PyAny>>| Python::with_gil(|py| {
                usize::extract(x.unwrap().as_ref(py)).unwrap()
            }))
            -> assert([0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    };
    assert_graphvis_snapshots!(hf);

    hf.run_available();
}

#[multiplatform_test(test)]
pub fn test_python_too_many_args() {
    let mut hf = hydroflow_syntax! {
        source_iter([(5,)])
            -> py_udf(r#"
def add(a, b):
    return a + b
            "#, "add")
            -> map(PyResult::<Py<PyAny>>::unwrap_err)
            -> map(|py_err| py_err.to_string())
            -> assert(["TypeError: add() missing 1 required positional argument: 'b'"]);
    };
    assert_graphvis_snapshots!(hf);

    hf.run_available();
}

#[multiplatform_test(test)]
pub fn test_python_two_args() {
    let mut hf = hydroflow_syntax! {
        source_iter([(5,1)])
            -> py_udf(r#"
def add(a, b):
    return a + b
            "#, "add")
            -> map(|x: PyResult<Py<PyAny>>| Python::with_gil(|py| {
                usize::extract(x.unwrap().as_ref(py)).unwrap()
            }))
            -> assert([6]);
    };
    assert_graphvis_snapshots!(hf);

    hf.run_available();
}
