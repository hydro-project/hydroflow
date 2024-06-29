use std::path::PathBuf;

#[derive(Clone)]
pub struct PerfOptions {
    pub output_file: PathBuf,
    pub frequency: u32,
}
