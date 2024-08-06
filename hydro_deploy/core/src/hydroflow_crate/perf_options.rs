use std::path::PathBuf;

#[derive(Clone, buildstructor::Builder)]
#[non_exhaustive] // Prevent direct construction.
pub struct PerfOptions {
    pub frequency: u32,

    /// TODO(mingwei): Only used by localhost, for now.
    pub perf_outfile: Option<PathBuf>,

    /// If set, what the write the folded output to.
    pub fold_outfile: Option<PathBuf>,
    pub fold_options: Option<inferno::collapse::perf::Options>,
    /// If set, what to write the output flamegraph SVG file to.
    pub flamegraph_outfile: Option<PathBuf>,
    // This type is super annoying and isn't `clone` and has a lifetime... so wrap in fn pointer for now.
    pub flamegraph_options: Option<fn() -> inferno::flamegraph::Options<'static>>,
}
