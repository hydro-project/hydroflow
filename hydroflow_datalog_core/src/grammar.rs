#[rust_sitter::grammar("datalog")]
#[allow(dead_code)]
pub mod datalog {
    #[rust_sitter::language]
    #[derive(Debug)]
    pub struct Program {
        pub rules: Vec<Declaration>,
    }

    #[derive(Debug, Clone)]
    pub enum Declaration {
        Input(#[rust_sitter::leaf(text = ".input")] (), Ident, RustSnippet),
        Output(
            #[rust_sitter::leaf(text = ".output")] (),
            Ident,
            RustSnippet,
        ),
        Async(
            #[rust_sitter::leaf(text = ".async")] (),
            Ident,
            /// A pipeline for data to be sent to another node, which must consume `(NodeID, Data)` pairs.
            RustSnippet,
            /// A pipeline for data received from another node, which must produce `Data` values.
            RustSnippet,
        ),
        Rule(Rule),
    }

    #[derive(Debug, Clone)]
    pub struct RustSnippet {
        #[rust_sitter::leaf(text = "`")]
        _start: (),
        #[rust_sitter::leaf(pattern = r#"[^`]*"#, transform = |s| s.to_string())]
        pub code: String,
        #[rust_sitter::leaf(text = "`")]
        _end: (),
    }

    #[derive(Debug, Clone)]
    pub struct Rule {
        pub target: TargetRelationExpr,

        pub rule_type: RuleType,

        #[rust_sitter::repeat(non_empty = true)]
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub sources: Vec<Atom>,

        #[rust_sitter::leaf(text = ".")]
        _dot: Option<()>,
    }

    #[derive(Debug, Clone)]
    pub enum RuleType {
        Sync(#[rust_sitter::leaf(text = ":-")] ()),
        NextTick(#[rust_sitter::leaf(text = ":+")] ()),
        Async(#[rust_sitter::leaf(text = ":~")] ()),
    }

    #[derive(Debug, Clone)]
    pub enum Atom {
        Relation(
            #[rust_sitter::leaf(text = "!")] Option<()>,
            InputRelationExpr,
        ),
        Predicate(PredicateExpr),
    }

    #[derive(Debug, Clone)]
    pub struct InputRelationExpr {
        pub name: Ident,

        #[rust_sitter::leaf(text = "(")]
        _l_paren: (),

        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub fields: Vec<Ident>,

        #[rust_sitter::leaf(text = ")")]
        _r_paren: (),
    }

    #[derive(Debug, Clone)]
    pub struct TargetRelationExpr {
        pub name: Ident,

        pub at_node: Option<AtNode>,

        #[rust_sitter::leaf(text = "(")]
        _l_paren: (),

        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        pub fields: Vec<TargetExpr>,

        #[rust_sitter::leaf(text = ")")]
        _r_paren: (),
    }

    #[derive(Debug, Clone)]
    #[allow(clippy::manual_non_exhaustive)]
    pub struct AtNode {
        #[rust_sitter::leaf(text = "@")]
        _at: (),

        pub node: TargetExpr,
    }

    #[derive(Debug, Clone)]
    pub enum TargetExpr {
        Ident(Ident),
        Aggregation(Aggregation),
    }

    impl TargetExpr {
        pub fn ident(&self) -> &Ident {
            match self {
                TargetExpr::Ident(ident) => ident,
                TargetExpr::Aggregation(a) => &a.ident,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Aggregation {
        pub tpe: AggregationType,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        pub ident: Ident,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
    }

    #[derive(Debug, Clone)]
    pub enum AggregationType {
        Max(#[rust_sitter::leaf(text = "max")] ()),
    }

    #[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Debug)]
    pub struct Ident {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |s| s.to_string())]
        pub name: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }

    #[derive(Debug, Clone)]
    pub enum BoolOp {
        Lt(#[rust_sitter::leaf(text = "<")] ()),
        LtEq(#[rust_sitter::leaf(text = "<=")] ()),
        Gt(#[rust_sitter::leaf(text = ">")] ()),
        GtEq(#[rust_sitter::leaf(text = ">=")] ()),
        Eq(#[rust_sitter::leaf(text = "==")] ()),
    }

    #[derive(Debug, Clone)]
    pub struct PredicateExpr {
        #[rust_sitter::leaf(text = "(")]
        _l_brace: (),

        pub left: Ident,
        pub op: BoolOp,
        pub right: Ident,

        #[rust_sitter::leaf(text = ")")]
        _r_brace: (),
    }
}
