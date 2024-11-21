use std::collections::{BTreeSet, HashSet};

mod prelude;
use prelude::is_prelude;
use quote::ToTokens;

#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<(HashSet<String>, HashSet<String>)>,
}

impl Default for ScopeStack {
    fn default() -> Self {
        ScopeStack {
            scopes: vec![(HashSet::new(), HashSet::new())],
        }
    }
}

impl ScopeStack {
    pub fn push(&mut self) {
        self.scopes.push((HashSet::new(), HashSet::new()));
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn insert_term(&mut self, ident: syn::Ident) {
        self.scopes
            .last_mut()
            .expect("Scope stack should not be empty")
            .0
            .insert(ident.to_string());
    }

    pub fn insert_type(&mut self, ident: syn::Ident) {
        self.scopes
            .last_mut()
            .expect("Scope stack should not be empty")
            .1
            .insert(ident.to_string());
    }

    pub fn contains_term(&self, ident: &syn::Ident) -> bool {
        let ident = ident.to_string();
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.0.contains(&ident))
    }

    pub fn contains_type(&self, ident: &syn::Ident) -> bool {
        let ident = ident.to_string();
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.1.contains(&ident))
    }
}

#[derive(Default)]
pub struct FreeVariableVisitor {
    pub free_variables: BTreeSet<syn::Ident>,
    pub current_scope: ScopeStack,
}

impl syn::visit_mut::VisitMut for FreeVariableVisitor {
    fn visit_expr_closure_mut(&mut self, i: &mut syn::ExprClosure) {
        self.current_scope.push();
        i.inputs.iter_mut().for_each(|input| {
            self.visit_pat_mut(input);
        });

        syn::visit_mut::visit_expr_closure_mut(self, i);

        self.current_scope.pop();
    }

    fn visit_item_fn_mut(&mut self, i: &mut syn::ItemFn) {
        self.current_scope.push();
        syn::visit_mut::visit_item_fn_mut(self, i);
        self.current_scope.pop();
    }

    fn visit_generic_param_mut(&mut self, i: &mut syn::GenericParam) {
        match i {
            syn::GenericParam::Type(type_param) => {
                self.current_scope.insert_type(type_param.ident.clone());
            }
            syn::GenericParam::Lifetime(lifetime_param) => {
                self.current_scope
                    .insert_type(lifetime_param.lifetime.ident.clone());
            }
            syn::GenericParam::Const(const_param) => {
                self.current_scope.insert_type(const_param.ident.clone());
            }
        }
    }

    fn visit_block_mut(&mut self, i: &mut syn::Block) {
        self.current_scope.push();
        syn::visit_mut::visit_block_mut(self, i);
        self.current_scope.pop();
    }

    fn visit_local_mut(&mut self, i: &mut syn::Local) {
        i.init.iter_mut().for_each(|init| {
            syn::visit_mut::visit_local_init_mut(self, init);
        });

        match &mut i.pat {
            syn::Pat::Ident(pat_ident) => {
                self.current_scope.insert_term(pat_ident.ident.clone());
            }
            syn::Pat::Type(pat_type) => {
                self.visit_pat_mut(&mut pat_type.pat);
            }
            syn::Pat::Wild(_) => {
                // Do nothing
            }
            syn::Pat::Tuple(pat_tuple) => {
                for el in &mut pat_tuple.elems {
                    self.visit_pat_mut(el);
                }
            }
            _ => panic!("Local variables must be identifiers, got {:?}", i.pat),
        }
    }

    fn visit_ident_mut(&mut self, i: &mut proc_macro2::Ident) {
        if !self.current_scope.contains_term(i) {
            self.free_variables.insert(i.clone());
            *i = syn::Ident::new(&format!("{}__free", i), i.span());
        }
    }

    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        if !self.current_scope.contains_type(&i.ident) {
            self.free_variables.insert(i.ident.clone());
            i.ident = syn::Ident::new(&format!("{}__free", i.ident), i.ident.span());
        }
    }

    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        if i.leading_colon.is_none() && !is_prelude(&i.segments.first().unwrap().ident) {
            let one_segment = i.segments.len() == 1;
            let node = i.segments.first_mut().unwrap();
            if one_segment && !self.current_scope.contains_term(&node.ident) {
                self.free_variables.insert(node.ident.clone());
                node.ident = syn::Ident::new(&format!("{}__free", node.ident), node.ident.span());
            }
        }

        for node in i.segments.iter_mut() {
            self.visit_path_arguments_mut(&mut node.arguments);
        }
    }

    fn visit_arm_mut(&mut self, i: &mut syn::Arm) {
        self.current_scope.push();
        syn::visit_mut::visit_arm_mut(self, i);
        self.current_scope.pop();
    }

    fn visit_field_pat_mut(&mut self, i: &mut syn::FieldPat) {
        for it in &mut i.attrs {
            self.visit_attribute_mut(it);
        }
        self.visit_pat_mut(&mut i.pat);
    }

    fn visit_pat_ident_mut(&mut self, i: &mut syn::PatIdent) {
        self.current_scope.insert_term(i.ident.clone());
    }

    fn visit_expr_method_call_mut(&mut self, i: &mut syn::ExprMethodCall) {
        syn::visit_mut::visit_expr_mut(self, &mut i.receiver);
        for arg in &mut i.args {
            self.visit_expr_mut(arg);
        }
    }

    fn visit_type_mut(&mut self, _: &mut syn::Type) {}

    fn visit_expr_struct_mut(&mut self, node: &mut syn::ExprStruct) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        if let Some(it) = &mut node.qself {
            self.visit_qself_mut(it);
        }
        // No need to capture the struct path
        // self.visit_path(&node.path);
        for el in syn::punctuated::Punctuated::pairs_mut(&mut node.fields) {
            let it = el.into_value();
            self.visit_expr_mut(&mut it.expr);
        }
        if let Some(it) = &mut node.rest {
            self.visit_expr_mut(it);
        }
    }

    fn visit_expr_field_mut(&mut self, i: &mut syn::ExprField) {
        self.visit_expr_mut(&mut i.base);
    }

    fn visit_macro_mut(&mut self, i: &mut syn::Macro) {
        // TODO(shadaj): emit a warning if our guess at parsing fails
        match i.delimiter {
            syn::MacroDelimiter::Paren(_binding_0) => {
                i.tokens = i
                    .parse_body_with(
                        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
                    )
                    .ok()
                    .map(|mut exprs| {
                        for arg in &mut exprs {
                            self.visit_expr_mut(arg);
                        }
                        exprs.to_token_stream()
                    })
                    .unwrap_or(i.tokens.clone());
            }
            syn::MacroDelimiter::Brace(_binding_0) => {
                i.tokens = i
                    .parse_body_with(syn::Block::parse_within)
                    .ok()
                    .map(|mut stmts| {
                        for stmt in &mut stmts {
                            self.visit_stmt_mut(stmt);
                        }
                        syn::punctuated::Punctuated::<syn::Stmt, syn::Token![;]>::from_iter(stmts)
                            .to_token_stream()
                    })
                    .unwrap_or(i.tokens.clone());
            }
            syn::MacroDelimiter::Bracket(_binding_0) => {
                i.tokens = i
                    .parse_body_with(
                        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
                    )
                    .ok()
                    .map(|mut exprs| {
                        for arg in &mut exprs {
                            self.visit_expr_mut(arg);
                        }
                        exprs.to_token_stream()
                    })
                    .unwrap_or(i.tokens.clone());
            }
        }
    }
}
