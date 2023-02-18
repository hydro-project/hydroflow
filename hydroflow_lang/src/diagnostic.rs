use proc_macro2::Span;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub span: Span,
    pub level: Level,
    pub message: String,
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
    pub fn is_error(&self) -> bool {
        self.level.is_error()
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
}
impl From<syn::Error> for Diagnostic {
    fn from(value: syn::Error) -> Self {
        Self::spanned(value.span(), Level::Error, value.to_string())
    }
}
