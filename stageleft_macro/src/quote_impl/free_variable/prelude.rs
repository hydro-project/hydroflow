use std::collections::HashSet;

use lazy_static::lazy_static;

lazy_static! {
    static ref PRELUDE: HashSet<&'static str> = {
        vec!["Some", "None", "Ok"]
            .into_iter()
            .collect::<HashSet<&'static str>>()
    };
}

pub fn is_prelude(ident: &syn::Ident) -> bool {
    let ident_str = ident.to_string();
    PRELUDE.contains(&ident_str.as_str())
}
