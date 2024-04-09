//! Utility methods for processing singleton references: `#my_var`.

use itertools::Itertools;
use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

use crate::parse::parse_terminated;

/// Finds all the singleton references `#my_var` and appends them to `found_idents`. Returns the
/// `TokenStream` but with the hashes removed from the varnames.
///
/// The returned tokens are used for "preflight" parsing, to check that the rest of the syntax is
/// OK. However the returned tokens are not used in the codegen as we need to use [`postprocess_singletons`]
/// later to substitute-in the context referencing code for each singleton
pub fn preprocess_singletons(tokens: TokenStream, found_idents: &mut Vec<Ident>) -> TokenStream {
    tokens
        .into_iter()
        .peekable()
        .batching(|iter| {
            let out = match iter.next()? {
                TokenTree::Group(group) => {
                    let mut new_group = Group::new(
                        group.delimiter(),
                        preprocess_singletons(group.stream(), found_idents),
                    );
                    new_group.set_span(group.span());
                    TokenTree::Group(new_group)
                }
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

/// Replaces singleton references `#my_var` with the code needed to actually get the value inside.
///
/// * `tokens` - The tokens to update singleton references within.
/// * `resolved_idents` - The context `StateHandle` varnames that correspond 1:1 and in the same
///   order as the singleton references within `tokens` (found in-order via [`preprocess_singletons`]).
pub fn postprocess_singletons(
    tokens: TokenStream,
    resolved_idents: impl IntoIterator<Item = Ident>,
) -> Punctuated<Expr, Token![,]> {
    let processed = postprocess_singletons_helper(tokens, resolved_idents.into_iter().by_ref());
    parse_terminated(processed).unwrap()
}

/// Internal recursive helper for [`postprocess_singletons`].
fn postprocess_singletons_helper(
    tokens: TokenStream,
    resolved_idents_iter: &mut impl Iterator<Item = Ident>,
) -> TokenStream {
    tokens
        .into_iter()
        .peekable()
        .batching(|iter| {
            let out = match iter.next()? {
                TokenTree::Group(group) => {
                    let mut new_group = Group::new(
                        group.delimiter(),
                        postprocess_singletons_helper(group.stream(), resolved_idents_iter),
                    );
                    new_group.set_span(group.span());
                    TokenTree::Group(new_group)
                }
                TokenTree::Ident(ident) => TokenTree::Ident(ident),
                TokenTree::Punct(punct) => {
                    if '#' == punct.as_char() && matches!(iter.peek(), Some(TokenTree::Ident(_))) {
                        // Found a singleton.
                        let _singleton_ident = iter.next();
                        let resolved_ident = resolved_idents_iter.next().unwrap();
                        TokenTree::Group(Group::new(
                            proc_macro2::Delimiter::Parenthesis,
                            quote! {
                                *context.state_ref(#resolved_ident).borrow_mut()
                            },
                        ))
                        // TokenTree::Ident(resolved_idents_iter.next().unwrap())
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
