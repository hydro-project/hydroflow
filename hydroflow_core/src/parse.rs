use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::{Pair, Punctuated};
use syn::spanned::Spanned;
use syn::token::{Bracket, Paren};
use syn::{bracketed, parenthesized, Expr, ExprPath, Ident, LitInt, Token};

pub struct HfCode {
    pub statements: Punctuated<HfStatement, Token![;]>,
}
impl Parse for HfCode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let statements = input.parse_terminated(HfStatement::parse)?;
        Ok(HfCode { statements })
    }
}
impl ToTokens for HfCode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.statements.to_tokens(tokens)
    }
}

pub enum HfStatement {
    Named(NamedHfStatement),
    Pipeline(Pipeline),
}
impl Parse for HfStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek2(Token![=]) {
            Ok(Self::Named(NamedHfStatement::parse(input)?))
        } else {
            Ok(Self::Pipeline(Pipeline::parse(input)?))
        }
    }
}
impl ToTokens for HfStatement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            HfStatement::Named(x) => x.to_tokens(tokens),
            HfStatement::Pipeline(x) => x.to_tokens(tokens),
        }
    }
}

pub struct NamedHfStatement {
    pub name: Ident,
    pub equals: Token![=],
    pub pipeline: Pipeline,
}
impl Parse for NamedHfStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let equals = input.parse()?;
        let pipeline = input.parse()?;
        Ok(Self {
            name,
            equals,
            pipeline,
        })
    }
}
impl ToTokens for NamedHfStatement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.to_tokens(tokens);
        self.equals.to_tokens(tokens);
        self.pipeline.to_tokens(tokens);
    }
}

pub enum Pipeline {
    Chain(ChainPipeline),
    Name(Ident),
    Operator(Operator),
}
impl Parse for Pipeline {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Paren) {
            Ok(Self::Chain(input.parse()?))
        } else if input.peek2(Paren) {
            Ok(Self::Operator(input.parse()?))
        } else {
            Ok(Self::Name(input.parse()?))
        }
    }
}
impl ToTokens for Pipeline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Pipeline::Chain(x) => x.to_tokens(tokens),
            Pipeline::Name(x) => x.to_tokens(tokens),
            Pipeline::Operator(x) => x.to_tokens(tokens),
        }
    }
}

pub struct ChainPipeline {
    pub paren_token: Paren,
    pub elems: Punctuated<Pipeline, ArrowConnector>,
}
impl Parse for ChainPipeline {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let elems: Punctuated<Pipeline, ArrowConnector> = Punctuated::parse_terminated(&content)?;

        match elems.pairs().next_back() {
            None => {
                elems.span().unwrap().error("Cannot have empty pipeline");
            }
            Some(Pair::Punctuated(_, trailing)) => {
                trailing.span().unwrap().error("Cannot have trailing arrow");
            }
            Some(Pair::End(_)) => {}
        }
        if elems.trailing_punct() {
            elems.span().unwrap().error("Cannot have empty pipeline");
        }

        Ok(Self { paren_token, elems })
    }
}
impl ToTokens for ChainPipeline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.paren_token.surround(tokens, |tokens| {
            self.elems.to_tokens(tokens);
        });
    }
}

pub struct ArrowConnector {
    pub src: Option<Indexing>,
    pub arrow: Token![->],
    pub dst: Option<Indexing>,
}
impl Parse for ArrowConnector {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut src = None;
        if input.peek(Bracket) {
            src = Some(input.parse()?);
        }
        let arrow = input.parse()?;
        let mut dst = None;
        if input.peek(Bracket) {
            dst = Some(input.parse()?);
        }
        Ok(Self { src, arrow, dst })
    }
}
impl ToTokens for ArrowConnector {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.src.to_tokens(tokens);
        self.arrow.to_tokens(tokens);
        self.dst.to_tokens(tokens);
    }
}

pub struct Indexing {
    pub bracket_token: Bracket,
    pub index: LitInt,
}
impl Parse for Indexing {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let bracket_token = bracketed!(content in input);
        let index = content.parse()?;
        Ok(Self {
            bracket_token,
            index,
        })
    }
}
impl ToTokens for Indexing {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.bracket_token.surround(tokens, |tokens| {
            self.index.to_tokens(tokens);
        });
    }
}

pub struct MultiplePipeline {
    pub name: Ident,
    pub bracket_token: Bracket,
    pub elems: Punctuated<Pipeline, Token![,]>,
}
impl Parse for MultiplePipeline {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        let content;
        let bracket_token = bracketed!(content in input);
        let mut elems = Punctuated::new();

        while !content.is_empty() {
            let first = content.parse()?;
            elems.push_value(first);
            if content.is_empty() {
                break;
            }
            let punct = content.parse()?;
            elems.push_punct(punct);
        }

        Ok(Self {
            name,
            bracket_token,
            elems,
        })
    }
}
impl ToTokens for MultiplePipeline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.bracket_token.surround(tokens, |tokens| {
            self.elems.to_tokens(tokens);
        });
    }
}

pub struct Operator {
    pub path: ExprPath,
    pub paren_token: Paren,
    pub args: Punctuated<Expr, Token![,]>,
}
impl Parse for Operator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);
        let mut args = Punctuated::new();

        while !content.is_empty() {
            let first = content.parse()?;
            args.push_value(first);
            if content.is_empty() {
                break;
            }
            let punct = content.parse()?;
            args.push_punct(punct);
        }

        Ok(Self {
            path,
            paren_token,
            args,
        })
    }
}
impl ToTokens for Operator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.path.to_tokens(tokens);
        self.paren_token.surround(tokens, |tokens| {
            self.args.to_tokens(tokens);
        });
    }
}
