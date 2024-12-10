mod clean;
mod map_ok_then;
mod pointer;
mod pre_push;
mod smudge;
mod splitter;

pub use clean::*;
pub use map_ok_then::*;
pub use pointer::*;
pub use pre_push::*;
pub use smudge::*;
pub use splitter::*;

use std::fmt::Display;

/// Creates a git ref underneath the namespace
/// `refs/gfs`
pub fn create_gfs_ref(id: impl Display) -> String {
    format!("refs/gfs/{id}")
}
