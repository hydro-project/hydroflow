use std::path::PathBuf;

#[derive(Clone, buildstructor::Builder)]
#[non_exhaustive] // Prevent direct construction.
pub struct PerfOptions {
    pub output_file: PathBuf,
    pub frequency: u32,
    pub fold_options: Option<inferno::collapse::perf::Options>,
    // This type is super annoying and isn't `clone` and has a lifetime... so wrap in fn pointer for now.
    pub flamegraph_options: Option<fn() -> inferno::flamegraph::Options<'static>>,
}
