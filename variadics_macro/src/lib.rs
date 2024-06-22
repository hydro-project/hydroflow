extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitInt};

struct InputLen {
    input: Ident,
    len: LitInt,
}

impl Parse for InputLen {
    fn parse(ts: ParseStream) -> syn::Result<Self> {
        let input = ts.parse()?;
        ts.parse::<syn::Token![,]>()?;
        let len = ts.parse()?;
        Ok(InputLen { input, len })
    }
}

fn gen_pattern(c: &str, remaining: usize, current: usize) -> proc_macro2::TokenStream {
    if remaining >= 1 {
        let identifier = format_ident!("{}", c.repeat(current));
        let inner = gen_pattern(c, remaining - 1, current + 1);
        quote! { (#identifier, #inner) }
    } else {
        quote! { () }
    }
}

fn gen_tuple(c: &str, remaining: usize, current: usize) -> proc_macro2::TokenStream {
    let identifier = format_ident!("{}", c.repeat(current));

    if remaining > 1 {
        // Continue to generate the next elements in the tuple.
        let inner = gen_tuple(c, remaining - 1, current + 1);
        if current == 1 {
            // At the first call, wrap all accumulated elements in a tuple.
            quote! { (#identifier, #inner) }
        } else {
            // Accumulate elements by appending them.
            quote! { #identifier, #inner }
        }
    } else {
        // The base case of recursion: return the last element.
        quote! { #identifier }
    }
}

#[proc_macro]
pub fn tuple(ts: TokenStream) -> TokenStream {
    let InputLen { input, len } = parse_macro_input!(ts as InputLen);
    let c = "x";
    let len = len.base10_parse::<usize>().unwrap();
    let pattern = gen_pattern(c, len, 1);
    let tuple = gen_tuple(c, len, 1);
    println!("tuple: {:?}", tuple);

    // Create the assignment statement
    let expanded = quote! {
        {
            let #pattern = #input;
            let retval = #tuple;
            retval
        }
    };

    expanded.into()
}
