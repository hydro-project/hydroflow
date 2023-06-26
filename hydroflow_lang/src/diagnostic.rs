use std::borrow::Cow;
use std::hash::{Hash, Hasher};

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote_spanned;
use serde::{Deserialize, Serialize};

use crate::pretty_span::PrettySpan;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Level {
    /// An error.
    ///
    /// The most severe and important diagnostic. Errors will prevent compilation.
    Error,
    /// A warning.
    ///
    /// The second most severe diagnostic. Will not stop compilation.
    Warning,
    /// A note.
    ///
    /// The third most severe, or second least severe diagnostic.
    Note,
    /// A help message.
    ///
    /// The least severe and important diagnostic.
    Help,
}
impl Level {
    pub fn is_error(&self) -> bool {
        self <= &Self::Error
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic<S = Span> {
    pub span: S,
    pub level: Level,
    pub message: String,
}
impl<S> Diagnostic<S> {
    pub fn is_error(&self) -> bool {
        self.level.is_error()
    }
}
impl Diagnostic {
    pub fn spanned(span: Span, level: Level, message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            span,
            level,
            message,
        }
    }

    pub fn emit(&self) {
        #[cfg(feature = "diagnostics")]
        {
            let pm_diag = match self.level {
                Level::Error => self.span.unwrap().error(&*self.message),
                Level::Warning => self.span.unwrap().warning(&*self.message),
                Level::Note => self.span.unwrap().note(&*self.message),
                Level::Help => self.span.unwrap().help(&*self.message),
            };
            pm_diag.emit();
        }
    }

    pub fn to_tokens(&self) -> TokenStream {
        let msg_lit: Literal = Literal::string(&self.message);
        let unique_ident = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            self.level.hash(&mut hasher);
            self.message.hash(&mut hasher);
            let hash = hasher.finish();
            Ident::new(&format!("diagnostic_{}", hash), self.span)
        };

        if Level::Error == self.level {
            quote_spanned! {self.span=>
                {
                    ::std::compile_error!(#msg_lit);
                }
            }
        } else {
            // Emit as a `#[deprecated]` warning message.
            let level_ident = Ident::new(&format!("{:?}", self.level), self.span);
            quote_spanned! {self.span=>
                {
                    #[allow(dead_code, non_snake_case)]
                    fn #unique_ident() {
                        #[deprecated = #msg_lit]
                        struct #level_ident {}
                        #[warn(deprecated)]
                        #level_ident {};
                    }
                }
            }
        }
    }

    pub fn to_serde(&self) -> Diagnostic<SerdeSpan> {
        let Self {
            span,
            level,
            message,
        } = self;
        Diagnostic {
            span: (*span).into(),
            level: *level,
            message: message.clone(),
        }
    }
}
impl From<syn::Error> for Diagnostic {
    fn from(value: syn::Error) -> Self {
        Self::spanned(value.span(), Level::Error, value.to_string())
    }
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}: {}", self.level, self.message)?;
        write!(f, "  --> {}", PrettySpan(self.span))?;
        Ok(())
    }
}
impl std::fmt::Display for Diagnostic<SerdeSpan> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}: {}", self.level, self.message)?;
        write!(f, "  --> {}", self.span)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerdeSpan {
    // https://github.com/serde-rs/serde/issues/1852#issuecomment-904840811
    #[serde(borrow)]
    pub path: Cow<'static, str>,
    pub line: usize,
    pub column: usize,
}
impl From<Span> for SerdeSpan {
    fn from(span: Span) -> Self {
        #[cfg(feature = "diagnostics")]
        let path = span
            .unwrap()
            .source_file()
            .path()
            .display()
            .to_string()
            .into();

        #[cfg(not(feature = "diagnostics"))]
        let path = "unknown".into();

        Self {
            path,
            line: span.start().line,
            column: span.start().column,
        }
    }
}
impl std::fmt::Display for SerdeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.path, self.line, self.column)
    }
}
