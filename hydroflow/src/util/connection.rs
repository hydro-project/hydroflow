use std::path::PathBuf;

use serde::{Serialize, Deserialize};

/// Describes a medium through which two HydroFlow services can communicate.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConnectionPipe {
    UnixSocket(PathBuf),
}
