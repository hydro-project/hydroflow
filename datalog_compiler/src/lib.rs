use quote::ToTokens;
use syn::parse_quote;

#[rust_sitter::grammar("datalog")]
#[allow(dead_code)]
mod datalog_grammar {
    #[rust_sitter::language]
    #[derive(Debug)]
    pub struct Program {
        rules: Vec<Declaration>,
    }

    #[derive(Debug)]
    pub enum Declaration {
        Input(#[rust_sitter::leaf(text = ".input")] (), Box<Ident>),
        Output(#[rust_sitter::leaf(text = ".output")] (), Box<Ident>),
        Rule(Box<Rule>),
    }

    #[derive(Debug)]
    pub struct Rule {
        target: Box<Target>,
        #[rust_sitter::leaf(text = ":-")]
        _from: (),
        #[rust_sitter::repeat(non_empty = true)]
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        sources: Vec<Target>,
    }

    #[derive(Debug)]
    pub struct Target {
        name: Box<Ident>,
        #[rust_sitter::leaf(text = "(")]
        _l_paren: (),
        #[rust_sitter::delimited(
            #[rust_sitter::leaf(text = ",")]
            ()
        )]
        fields: Vec<Ident>,
        #[rust_sitter::leaf(text = ")")]
        _r_paren: (),
    }

    #[derive(Debug)]
    pub struct Ident {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |s| s.to_string())]
        pub name: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

fn gen_datalog_program(literal: proc_macro2::Literal) -> syn::Lit {
    let str_node: syn::LitStr = parse_quote!(#literal);
    let actual_str = str_node.value();
    let program = datalog_grammar::parse(&actual_str).unwrap();
    let program_tree = format!("{:?}", program);
    syn::parse_quote!(#program_tree)
}

#[proc_macro]
pub fn datalog(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = proc_macro2::TokenStream::from(item);
    let literal: proc_macro2::Literal = syn::parse_quote! {
        #item
    };
    proc_macro::TokenStream::from(gen_datalog_program(literal).to_token_stream())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{Read, Write};
    use std::process::Command;

    use quote::ToTokens;
    use syn::parse_quote;
    use tempfile::tempdir;

    use super::gen_datalog_program;

    fn rustfmt_code(code: &str) -> String {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("temp.rs");
        let mut file = File::create(file_path.clone()).unwrap();

        writeln!(file, "{}", code).unwrap();
        drop(file);

        Command::new("rustfmt")
            .arg(file_path.to_str().unwrap())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        let mut file = File::open(file_path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        drop(file);
        dir.close().unwrap();
        data
    }

    #[test]
    fn minimal_program() {
        insta::assert_display_snapshot!(rustfmt_code(
            &gen_datalog_program(parse_quote!(
                r#"
                .input edge
                .output path

                path(x, y) :- edge(x, y)
                path(x, y) :- path(x, z), edge(z, y)
            "#
            ))
            .to_token_stream()
            .to_string()
        ));
    }
}
