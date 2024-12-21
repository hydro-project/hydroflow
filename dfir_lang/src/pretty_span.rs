//! Pretty, human-readable printing of [`proc_macro2::Span`]s.

extern crate proc_macro;

/// Helper struct which displays the span as `path:row:col` for human reading/IDE linking.
/// Example: `dfir\tests\surface_syntax.rs:42:18`.
pub struct PrettySpan(pub proc_macro2::Span);
impl std::fmt::Display for PrettySpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(nightly)]
        if proc_macro::is_available() {
            let span = self.0.unwrap();
            write!(
                f,
                "{}:{}:{}",
                span.source_file().path().display(),
                span.start().line(),
                span.start().column(),
            )?;
            return Ok(());
        }

        write!(
            f,
            "unknown:{}:{}",
            self.0.start().line,
            self.0.start().column
        )
    }
}

/// Helper struct which displays the span as `row:col` for human reading.
pub struct PrettyRowCol(pub proc_macro2::Span);
impl std::fmt::Display for PrettyRowCol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let span = self.0;
        write!(f, "{}:{}", span.start().line, span.start().column)
    }
}
