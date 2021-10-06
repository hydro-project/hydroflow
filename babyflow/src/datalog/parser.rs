use anyhow::{anyhow, bail};

use super::lang::{Datum, Expr};

#[derive(Debug, Clone)]
pub struct Predicate {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Clause {
    pub head: Predicate,
    pub body: Vec<Predicate>,
}

#[derive(Debug, Clone)]
pub struct Syntax {
    pub clauses: Vec<Clause>,
}

#[derive(Debug)]
struct Parser {
    chars: Vec<char>,
    idx: usize,
}

impl Parser {
    fn parse(&mut self) -> anyhow::Result<Syntax> {
        let mut clauses = Vec::new();
        self.munch();
        while self.idx < self.chars.len() {
            clauses.push(self.clause()?);
            self.munch();
        }

        Ok(Syntax { clauses })
    }

    fn munch(&mut self) {
        while self.idx < self.chars.len() && self.chars[self.idx].is_ascii_whitespace() {
            self.idx += 1;
        }
    }

    fn clause(&mut self) -> anyhow::Result<Clause> {
        let head = self.predicate()?;
        self.munch();
        if self.peek() == Some('.') {
            self.expect(".")?;
            Ok(Clause {
                head,
                body: Vec::new(),
            })
        } else {
            self.expect("<-")?;
            self.munch();
            let mut body = vec![self.predicate()?];
            self.munch();
            while self.peek() == Some(',') {
                self.expect(",")?;
                self.munch();
                body.push(self.predicate()?);
                self.munch();
            }
            self.expect(".")?;
            Ok(Clause { head, body })
        }
    }

    fn predicate(&mut self) -> anyhow::Result<Predicate> {
        let name = self.word()?;
        self.munch();
        self.expect("(")?;
        self.munch();
        let mut args = vec![self.expr()?];
        self.munch();
        while self.peek() == Some(',') {
            self.expect(",")?;
            self.munch();
            args.push(self.expr()?);
            self.munch();
        }
        self.expect(")")?;
        Ok(Predicate { name, args })
    }

    fn peek(&self) -> Option<char> {
        if self.idx < self.chars.len() {
            Some(self.chars[self.idx])
        } else {
            None
        }
    }

    fn expr(&mut self) -> anyhow::Result<Expr> {
        if self.idx >= self.chars.len() {
            bail!("expected expr");
        }
        if self.chars[self.idx].is_alphabetic() {
            let name = self.word()?;
            if name.chars().next().unwrap().is_uppercase() {
                Ok(Expr::Var(name))
            } else {
                Ok(Expr::Datum(Datum::Atom(name)))
            }
        } else if self.chars[self.idx].is_numeric() {
            let mut s = String::new();
            while self.idx < self.chars.len() && self.chars[self.idx].is_numeric() {
                s.push(self.chars[self.idx]);
                self.idx += 1;
            }
            Ok(Expr::Datum(Datum::Int(s.parse()?)))
        } else {
            Err(anyhow!("couldn't parse from {}", self.chars[self.idx]))
        }
    }

    fn expect(&mut self, s: &str) -> anyhow::Result<()> {
        for ch in s.chars() {
            if self.idx >= self.chars.len() || self.chars[self.idx] != ch {
                println!("{:?} {}", self, s);
                bail!("expected {}", s);
            }
            self.idx += 1;
        }
        Ok(())
    }

    fn word(&mut self) -> anyhow::Result<String> {
        let mut out = String::new();
        while self.idx < self.chars.len() && self.chars[self.idx].is_alphanumeric() {
            out.push(self.chars[self.idx]);
            self.idx += 1;
        }

        Ok(out)
    }
}

pub fn parse(s: &str) -> anyhow::Result<Syntax> {
    let mut p = Parser {
        idx: 0,
        chars: s.chars().collect(),
    };

    p.parse()
}

#[test]
fn test_parse() {
    use datadriven::walk;

    walk("src/testdata/parse", |f| {
        f.run(|test_case| format!("{:#?}\n", parse(&test_case.input).unwrap()))
    })
}
