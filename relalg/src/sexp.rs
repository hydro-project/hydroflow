use anyhow::bail;

#[derive(Debug, Clone)]
struct Parser {
    input: Vec<char>,
    idx: usize,
}

fn closer(ch: char) -> char {
    match ch {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!(""),
    }
}

impl Parser {
    fn atom_char(ch: char) -> bool {
        ch != '{'
            && ch != '}'
            && ch != '['
            && ch != ']'
            && ch != '('
            && ch != ')'
            && !ch.is_whitespace()
    }

    fn new(input: String) -> Self {
        Parser {
            input: input.chars().collect(),
            idx: 0,
        }
    }

    fn munch(&mut self) {
        while self.idx < self.input.len() && self.input[self.idx].is_whitespace() {
            self.idx += 1;
        }
    }

    pub fn parse_multiple(&mut self) -> Result<Vec<Sexp>, anyhow::Error> {
        let mut out = Vec::new();
        loop {
            self.munch();
            if self.idx >= self.input.len() {
                break;
            }
            match self.input[self.idx] {
                ')' | ']' | '}' => break,
                _ => out.push(self.parse_one()?),
            }
        }

        Ok(out)
    }

    fn parse_one(&mut self) -> Result<Sexp, anyhow::Error> {
        self.munch();
        match self.input[self.idx] {
            ch @ '[' | ch @ '(' | ch @ '{' => {
                self.idx += 1;
                let contents = self.parse_multiple()?;
                self.munch();
                if self.idx >= self.input.len() || self.input[self.idx] != closer(ch) {
                    bail!("unclosed {}", ch);
                }
                self.idx += 1;
                Ok(Sexp::List(contents, ch))
            }
            '"' => {
                self.idx += 1;
                let mut out = String::new();
                while self.idx < self.input.len() {
                    match self.input[self.idx] {
                        '"' => {
                            self.idx += 1;
                            return Ok(Sexp::String(out));
                        }
                        '\\' => {
                            self.idx += 1;
                            if self.idx >= self.input.len() {
                                bail!("unexpected EOF after \\")
                            }
                            match self.input[self.idx] {
                                'n' => out.push('\n'),
                                't' => out.push('\t'),
                                '"' => out.push('"'),
                                '\\' => out.push('\\'),
                                _ => bail!("unknown escape sequence"),
                            }
                            self.idx += 1;
                        }
                        ch => {
                            out.push(ch);
                        }
                    }
                    self.idx += 1;
                }
                bail!("unclosed '\"'");
            }
            ch if Self::atom_char(ch) => {
                let mut s = String::new();
                while self.idx < self.input.len() && Self::atom_char(self.input[self.idx]) {
                    s.push(self.input[self.idx]);
                    self.idx += 1;
                }
                Ok(Sexp::Atom(s))
            }
            _ => bail!("unhandled: {:?}", self.input[self.idx]),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Sexp {
    Atom(String),
    List(Vec<Sexp>, char),
    String(String),
}

impl Sexp {
    pub fn expect_atom(self) -> Result<String, anyhow::Error> {
        match self {
            Sexp::Atom(r) => Ok(r),
            _ => bail!("expected atom"),
        }
    }

    pub fn expect_list(self) -> Result<(Vec<Sexp>, char), anyhow::Error> {
        match self {
            Sexp::List(v, ch) => Ok((v, ch)),
            _ => bail!("expected list"),
        }
    }

    pub fn expect_string(self) -> Result<String, anyhow::Error> {
        match self {
            Sexp::String(r) => Ok(r),
            _ => bail!("expected string"),
        }
    }

    pub fn parse(s: String) -> Result<Vec<Sexp>, anyhow::Error> {
        let mut p = Parser::new(s);
        p.parse_multiple()
    }

    pub fn format(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    fn write(&self, s: &mut String) {
        match self {
            Sexp::String(st) => {
                s.push('"');
                for ch in st.chars() {
                    match ch {
                        '\n' => s.push_str("\\n"),
                        '"' => s.push_str("\\\""),
                        '\t' => s.push_str("\\t"),
                        '\\' => s.push_str("\\\\"),
                        ch => s.push(ch),
                    }
                }
                s.push('"');
            }
            Sexp::Atom(atom) => s.push_str(atom),
            Sexp::List(v, ch) => {
                s.push(*ch);
                let mut space = "";
                for sexp in v {
                    s.push_str(space);
                    space = " ";
                    Self::write(sexp, s);
                }
                s.push(closer(*ch));
            }
        }
    }
}

pub trait FromSexp: Sized {
    fn from_sexp(s: Sexp) -> Result<Self, anyhow::Error>;
}

impl FromSexp for Sexp {
    fn from_sexp(s: Sexp) -> Result<Self, anyhow::Error> {
        Ok(s)
    }
}

impl ToSexp for Sexp {
    fn to_sexp(&self) -> Sexp {
        self.clone()
    }
}

pub trait ToSexp {
    fn to_sexp(&self) -> Sexp;
}

impl<I: FromSexp> FromSexp for Vec<I> {
    fn from_sexp(s: Sexp) -> Result<Self, anyhow::Error> {
        let (v, _) = s.expect_list()?;
        v.into_iter()
            .map(|s| I::from_sexp(s))
            .collect::<Result<Self, _>>()
    }
}

impl<I: ToSexp> ToSexp for Vec<I> {
    fn to_sexp(&self) -> Sexp {
        Sexp::List(self.iter().map(|i| i.to_sexp()).collect(), '[')
    }
}

impl<A: FromSexp, B: FromSexp> FromSexp for (A, B) {
    fn from_sexp(s: Sexp) -> Result<Self, anyhow::Error> {
        let (v, _) = s.expect_list()?;
        if v.len() != 2 {
            bail!("expected pair");
        }
        // TODO: remove clone
        Ok((A::from_sexp(v[0].clone())?, B::from_sexp(v[1].clone())?))
    }
}

impl<A: ToSexp, B: ToSexp> ToSexp for (A, B) {
    fn to_sexp(&self) -> Sexp {
        Sexp::List(vec![self.0.to_sexp(), self.1.to_sexp()], '(')
    }
}

macro_rules! impl_from_fromstr {
    ($name:ident) => {
        impl FromSexp for $name {
            fn from_sexp(s: Sexp) -> Result<Self, anyhow::Error> {
                Ok(s.expect_atom()?.parse::<$name>()?)
            }
        }

        impl ToSexp for $name {
            fn to_sexp(&self) -> Sexp {
                Sexp::Atom(format!("{}", self))
            }
        }
    };
}

impl_from_fromstr!(i64);
impl_from_fromstr!(usize);
impl_from_fromstr!(String);

#[derive(Debug, Clone)]
struct LitString(String);

impl FromSexp for LitString {
    fn from_sexp(s: Sexp) -> Result<Self, anyhow::Error> {
        Ok(LitString(s.expect_string()?.parse::<String>()?))
    }
}

impl ToSexp for LitString {
    fn to_sexp(&self) -> Sexp {
        Sexp::String(self.0.to_string())
    }
}

impl ToSexp for () {
    fn to_sexp(&self) -> Sexp {
        Sexp::List(Vec::new(), '(')
    }
}

#[test]
fn test_parse() {
    for s in ["1", "(1)", "(plus (one two))", "(+ 1 2)", "\"foo\""] {
        assert_eq!(
            s,
            Sexp::parse(s.into())
                .unwrap()
                .into_iter()
                .map(|s| s.format())
                .collect::<Vec<String>>()
                .join(" ")
        );
    }
}
