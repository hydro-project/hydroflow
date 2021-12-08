use relalg::{Datum, ScalarExpr};

// Can copy-paste the examples from the tests in here to actually run them.

fn main() {
    let __values_2 = vec![
        vec![
            ScalarExpr::Literal(Datum::Int(1i64)).eval(&Vec::new()),
            ScalarExpr::Literal(Datum::Int(2i64)).eval(&Vec::new()),
            ScalarExpr::Literal(Datum::Int(3i64)).eval(&Vec::new()),
        ],
        vec![
            ScalarExpr::Literal(Datum::Int(4i64)).eval(&Vec::new()),
            ScalarExpr::Literal(Datum::Int(5i64)).eval(&Vec::new()),
            ScalarExpr::Literal(Datum::Int(6i64)).eval(&Vec::new()),
        ],
    ]
    .into_iter();
    let __project_1 = __values_2.map(|row| {
        vec![
            ScalarExpr::Plus(
                Box::new(ScalarExpr::ColRef(0usize)),
                Box::new(ScalarExpr::Literal(Datum::Int(1i64))),
            )
            .eval(&row),
            ScalarExpr::Plus(
                Box::new(ScalarExpr::Literal(Datum::Int(5i64))),
                Box::new(ScalarExpr::Plus(
                    Box::new(ScalarExpr::ColRef(1usize)),
                    Box::new(ScalarExpr::ColRef(2usize)),
                )),
            )
            .eval(&row),
        ]
    });
    for row in __project_1 {
        println!("{:?}", row);
    }
}
