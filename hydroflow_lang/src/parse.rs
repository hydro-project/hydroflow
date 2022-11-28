use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Paren};
use syn::{
    bracketed, parenthesized, Expr, GenericArgument, Ident, Path, PathArguments, PathSegment, Token,
};

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
    Paren(PipelineParen),
    Link(PipelineLink),
    Name(Ident),
    Operator(Operator),
}
impl Pipeline {
    fn parse_helper(input: ParseStream) -> syn::Result<Self> {
        let lhs = Self::parse_one(input)?;
        if input.is_empty() || input.peek(Token![;]) {
            Ok(lhs)
        } else {
            let connector = input.parse()?;
            let rhs = input.parse()?;
            Ok(Self::Link(PipelineLink {
                lhs: Box::new(lhs),
                connector,
                rhs,
            }))
        }
    }

    fn parse_one(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Paren) {
            Ok(Self::Paren(input.parse()?))
        } else if input.peek2(Paren) || input.peek2(Token![<]) || input.peek2(Token![::]) {
            Ok(Self::Operator(input.parse()?))
        } else {
            Ok(Self::Name(input.parse()?))
        }
    }
}
impl Parse for Pipeline {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::parse_helper(input)
    }
}
impl ToTokens for Pipeline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Pipeline::Paren(x) => x.to_tokens(tokens),
            Pipeline::Link(x) => x.to_tokens(tokens),
            Pipeline::Name(x) => x.to_tokens(tokens),
            Pipeline::Operator(x) => x.to_tokens(tokens),
        }
    }
}

pub struct PipelineParen {
    pub paren_token: Paren,
    pub pipeline: Box<Pipeline>,
}
impl Parse for PipelineParen {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let pipeline = content.parse()?;
        Ok(Self {
            paren_token,
            pipeline,
        })
    }
}
impl ToTokens for PipelineParen {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.paren_token.surround(tokens, |tokens| {
            self.pipeline.to_tokens(tokens);
        });
    }
}

pub struct PipelineLink {
    pub lhs: Box<Pipeline>,
    pub connector: ArrowConnector,
    pub rhs: Box<Pipeline>,
}
impl Parse for PipelineLink {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lhs = input.parse()?;
        let connector = input.parse()?;
        let rhs = input.parse()?;

        Ok(Self {
            lhs,
            connector,
            rhs,
        })
    }
}
impl ToTokens for PipelineLink {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lhs.to_tokens(tokens);
        self.connector.to_tokens(tokens);
        self.rhs.to_tokens(tokens);
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
    pub index: TokenStream,
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

pub struct Operator {
    pub path: Path,
    pub paren_token: Paren,
    pub args: Punctuated<Expr, Token![,]>,
}
impl Operator {
    pub fn name(&self) -> Path {
        Path {
            leading_colon: self.path.leading_colon,
            segments: self
                .path
                .segments
                .iter()
                .map(|seg| PathSegment {
                    ident: seg.ident.clone(),
                    arguments: PathArguments::None,
                })
                .collect(),
        }
    }

    pub fn name_string(&self) -> String {
        self.name().to_token_stream().to_string()
    }

    pub fn type_arguments(&self) -> Option<&Punctuated<GenericArgument, Token![,]>> {
        let end = self.path.segments.last()?;
        if let PathArguments::AngleBracketed(type_args) = &end.arguments {
            Some(&type_args.args)
        } else {
            None
        }
    }

    pub fn args(&self) -> &Punctuated<Expr, Token![,]> {
        &self.args
    }
}
impl Parse for Operator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);
        let args = Punctuated::parse_terminated(&content)?;

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
