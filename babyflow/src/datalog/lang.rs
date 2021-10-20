#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Datum {
    Int(i64),
    Atom(String),
}

impl std::fmt::Display for Datum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Datum::Int(i) => write!(f, "{}", i),
            Datum::Atom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var(String),
    Datum(Datum),
}
