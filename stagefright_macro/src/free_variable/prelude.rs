use std::collections::HashSet;

use lazy_static::lazy_static;

lazy_static! {
    static ref PRELUDE: HashSet<&'static str> = {
        vec![
            // https://doc.rust-lang.org/core/
            "bool",
            "char",
            "f32",
            "f64",
            "i8",
            "i16",
            "i32",
            "i64",
            "i128",
            "isize",
            "str",
            "u8",
            "u16",
            "u32",
            "u64",
            "u128",
            "usize",
            // https://doc.rust-lang.org/std/prelude/index.html
            "Copy",
            "Send",
            "Sized",
            "Sync",
            "Unpin",
            "Drop",
            "Fn",
            "FnMut",
            "FnOnce",
            "drop",
            "Box",
            "ToOwned",
            "Clone",
            "PartialEq",
            "PartialOrd",
            "Eq",
            "Ord",
            "AsRef",
            "AsMut",
            "Into",
            "From",
            "Default",
            "Iterator",
            "Extend",
            "IntoIterator",
            "DoubleEndedIterator",
            "ExactSizeIterator",
            "Option",
            "Some",
            "None",
            "Result",
            "Ok",
            "Err",
            "String",
            "ToString",
            "Vec",
        ]
        .into_iter()
        .collect::<HashSet<&'static str>>()
    };
}

pub fn is_prelude(ident: &syn::Ident) -> bool {
    let ident_str = ident.to_string();
    PRELUDE.contains(&ident_str.as_str())
}
