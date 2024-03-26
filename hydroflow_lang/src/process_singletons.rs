use itertools::Itertools;
use proc_macro2::{Group, Ident, TokenStream, TokenTree};

pub fn preprocess_singletons(tokens: TokenStream, found_idents: &mut Vec<Ident>) -> TokenStream {
    tokens
        .into_iter()
        .peekable()
        .batching(|iter| {
            let out = match iter.next()? {
                TokenTree::Group(group) => TokenTree::Group(Group::new(
                    group.delimiter(),
                    preprocess_singletons(group.stream(), found_idents),
                )),
                TokenTree::Ident(ident) => TokenTree::Ident(ident),
                TokenTree::Punct(punct) => {
                    if '#' == punct.as_char() && matches!(iter.peek(), Some(TokenTree::Ident(_))) {
                        // Found a singleton.
                        let Some(TokenTree::Ident(singleton_ident)) = iter.next() else {
                            unreachable!()
                        };
                        found_idents.push(singleton_ident.clone());
                        TokenTree::Ident(singleton_ident)
                    } else {
                        TokenTree::Punct(punct)
                    }
                }
                TokenTree::Literal(lit) => TokenTree::Literal(lit),
            };
            Some(out)
        })
        .collect()
}

pub fn postprocess_singletons(tokens: TokenStream, resolved_idents: &[Ident]) -> TokenStream {
    postprocess_singletons_helper(tokens, resolved_idents.iter().cloned().by_ref())
}

fn postprocess_singletons_helper(
    tokens: TokenStream,
    resolved_idents_iter: &mut impl Iterator<Item = Ident>,
) -> TokenStream {
    tokens
        .into_iter()
        .peekable()
        .batching(|iter| {
            let out = match iter.next()? {
                TokenTree::Group(group) => TokenTree::Group(Group::new(
                    group.delimiter(),
                    postprocess_singletons_helper(group.stream(), resolved_idents_iter),
                )),
                TokenTree::Ident(ident) => TokenTree::Ident(ident),
                TokenTree::Punct(punct) => {
                    if '#' == punct.as_char() && matches!(iter.peek(), Some(TokenTree::Ident(_))) {
                        // Found a singleton.
                        let _singleton_ident = iter.next();
                        TokenTree::Ident(resolved_idents_iter.next().unwrap())
                    } else {
                        TokenTree::Punct(punct)
                    }
                }
                TokenTree::Literal(lit) => TokenTree::Literal(lit),
            };
            Some(out)
        })
        .collect()
}
