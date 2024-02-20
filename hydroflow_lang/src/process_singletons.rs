use itertools::Itertools;
use proc_macro2::{Group, Ident, TokenStream, TokenTree};

pub fn preprocess_singletons(tokens: TokenStream, found_idents: &mut Vec<Ident>) -> TokenStream {
    let mut hash_found = false;
    tokens
        .into_iter()
        .peekable()
        .batching(|iter| {
            let out = match iter.next()? {
                TokenTree::Group(group) => TokenTree::Group(Group::new(
                    group.delimiter(),
                    preprocess_singletons(group.stream(), found_idents),
                )),
                TokenTree::Ident(ident) => {
                    if std::mem::replace(&mut hash_found, false) {
                        found_idents.push(ident.clone());
                    }
                    TokenTree::Ident(ident)
                }
                TokenTree::Punct(punct) => {
                    if '#' == punct.as_char() && matches!(iter.peek(), Some(TokenTree::Ident(_))) {
                        // Found a singleton.
                        let Some(TokenTree::Ident(ident)) = iter.next() else {
                            unreachable!()
                        };
                        hash_found = true;
                        found_idents.push(ident.clone());
                        TokenTree::Ident(ident)
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
