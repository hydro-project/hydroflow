/// Helper struct which displays the span as `path:row:col` for human reading/IDE linking.
/// Example: `hydroflow\tests\surface_syntax.rs:42:18`.
pub struct PrettySpan(pub proc_macro2::Span);
impl std::fmt::Display for PrettySpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let span = self.0.unwrap();
            write!(
                f,
                "{}:{}:{}",
                span.source_file().path().display(),
                span.start().line,
                span.start().column
            )
        }

        #[cfg(target_arch = "wasm32")]
        {
            write!(f, "unknown")
        }
    }
}

/// Helper struct which displays the span as `row:col` for human reading.
pub struct PrettyRowCol(pub proc_macro2::Span);
impl std::fmt::Display for PrettyRowCol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let span = self.0.unwrap();
            write!(f, "{}:{}", span.start().line, span.start().column)
        }

        #[cfg(target_arch = "wasm32")]
        {
            write!(f, "unknown")
        }
    }
}
