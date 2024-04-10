use proc_macro2::Span;
use syn::parse_quote;
use syn::visit_mut::VisitMut;

use crate::runtime_support::get_final_crate_name;

/// Rewrites use of alloc::string::* to use std::string::*
struct RewriteAlloc {
    mapping: Option<(String, String)>,
}

impl VisitMut for RewriteAlloc {
    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        if i.segments.iter().take(2).collect::<Vec<_>>()
            == vec![
                &syn::PathSegment::from(syn::Ident::new("alloc", Span::call_site())),
                &syn::PathSegment::from(syn::Ident::new("string", Span::call_site())),
            ]
        {
            *i.segments.first_mut().unwrap() =
                syn::PathSegment::from(syn::Ident::new("std", Span::call_site()));
        } else if let Some((macro_name, final_name)) = &self.mapping {
            if i.segments.first().unwrap().ident == macro_name {
                *i.segments.first_mut().unwrap() =
                    syn::parse2(get_final_crate_name(final_name)).unwrap();

                i.segments.insert(1, parse_quote!(__staged));
            }
        }
    }
}

/// Captures a fully qualified path to a given type, which is useful when
/// the generated code needs to explicitly refer to a type known at staging time.
///
/// This API is fairly experimental, and comes with caveats. For example, it cannot
/// handle closure types. In addition, when a user refers to a re-exported type,
/// the original type path may be returned here, which could involve private modules.
///
/// Also, users must be careful to ensure that any crates referred in the type are
/// available where it is spliced.
pub fn quote_type<T>() -> syn::Type {
    let mut t_type: syn::Type = syn::parse_str(std::any::type_name::<T>()).unwrap();
    let mapping = super::runtime_support::MACRO_TO_CRATE.with(|m| m.borrow().clone());
    RewriteAlloc { mapping }.visit_type_mut(&mut t_type);

    t_type
}
