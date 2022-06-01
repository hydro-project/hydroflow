use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, Block};

#[proc_macro]
pub fn hydroflow_parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let input: Block = parse_quote!({ #input });
    input.into_token_stream().into()
}
