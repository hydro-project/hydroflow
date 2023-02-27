use std::iter::Iterator;

use proc_macro::TokenStream;
use quote::quote;

#[derive(Clone, Copy, Debug)]
enum Platform {
    Default,
    Wasm,
}

const DEFAULT_PLATFORMS: [Platform; 2] = [Platform::Default, Platform::Wasm];

#[proc_macro_attribute]
pub fn multiplatform_test(attr: TokenStream, body: TokenStream) -> TokenStream {
    let ts = multiplatform_test_impl(
        proc_macro2::TokenStream::from(attr),
        proc_macro2::TokenStream::from(body),
    );
    TokenStream::from(ts)
}

fn multiplatform_test_impl(
    attr: proc_macro2::TokenStream,
    body: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let mut attr = attr.into_iter();
    let mut platforms = Vec::<Platform>::new();

    while let Some(token) = attr.next() {
        let platform = match &token {
            proc_macro2::TokenTree::Ident(i) => match i.to_string().as_str() {
                "default" => Platform::Default,
                "wasm" => Platform::Wasm,
                other => panic!(
                    "unknown platform {}; want platform in [default, wasm]",
                    other
                ),
            },
            _ => panic!("malformed #[multiplatform_test] attribute; expected identifier"),
        };
        platforms.push(platform);

        match &attr.next() {
            Some(proc_macro2::TokenTree::Punct(op)) if op.as_char() == ',' => {}
            Some(_) => panic!("malformed `#[multiplatform_test]` attribute; expected `,`"),
            None => break,
        }
    }

    let mut output = Vec::<proc_macro2::TokenStream>::new();

    if platforms.is_empty() {
        platforms.extend(DEFAULT_PLATFORMS.iter());
    }

    for p in platforms {
        let tokens = match p {
            Platform::Default => quote! { #[test] },
            // Fully specify wasm_bindgen_test so that the consumer does not need to add another
            // use statement. They still need to depend on the wasm_bindgen_test_macro crate,
            // though.
            Platform::Wasm => quote! { #[wasm_bindgen_test_macro::wasm_bindgen_test] },
        };
        output.push(tokens);
    }

    output.push(body);

    output.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_platforms() {
        let test_fn = quote! { fn test() { } };
        let attrs = proc_macro2::TokenStream::new();
        let got = multiplatform_test_impl(attrs, test_fn);
        let want = quote! {
            #[test]
            #[wasm_bindgen_test_macro::wasm_bindgen_test]
            fn test() { }
        };

        assert_eq!(want.to_string(), got.to_string());
    }

    #[test]
    fn test_host_platform() {
        let test_fn = quote! { fn test() { } };
        let attrs = quote! { default };
        let got = multiplatform_test_impl(attrs, test_fn);
        let want = quote! {
            #[test]
            fn test() { }
        };

        assert_eq!(want.to_string(), got.to_string());
    }

    #[test]
    fn test_wasm_platform() {
        let test_fn = quote! { fn test() { } };
        let attrs = quote! { wasm };
        let got = multiplatform_test_impl(attrs, test_fn);
        let want = quote! {
            #[wasm_bindgen_test_macro::wasm_bindgen_test]
            fn test() { }
        };

        assert_eq!(want.to_string(), got.to_string());
    }

    #[test]
    fn test_host_wasm_platform() {
        let test_fn = quote! { fn test() { } };
        let attrs = quote! { default, wasm };
        let got = multiplatform_test_impl(attrs, test_fn);
        let want = quote! {
            #[test]
            #[wasm_bindgen_test_macro::wasm_bindgen_test]
            fn test() { }
        };

        assert_eq!(want.to_string(), got.to_string());
    }

    #[test]
    #[should_panic(expected = "unknown platform")]
    fn test_unknown_platform() {
        let test_fn = quote! { fn test() { } };
        let attrs = quote! { hello };
        multiplatform_test_impl(attrs, test_fn);
    }

    #[test]
    #[should_panic(expected = "expected `,`")]
    fn test_invalid_attr_nocomma_platform() {
        let test_fn = quote! { fn test() { } };
        let attrs = quote! { wasm() };
        multiplatform_test_impl(attrs, test_fn);
    }

    #[test]
    #[should_panic(expected = "expected identifier")]
    fn test_invalid_attr_noident_platform() {
        let test_fn = quote! { fn test() { } };
        let attrs = quote! { () };
        multiplatform_test_impl(attrs, test_fn);
    }
}
