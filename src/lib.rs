mod clean;
mod pointer;
mod pre_push;
mod smudge;
mod splitter;
mod traits;

pub use clean::*;
pub use pointer::*;
pub use pre_push::*;
pub use smudge::*;
pub use splitter::*;
pub use traits::*;

use std::fmt::Display;

/// Creates a git ref underneath the namespace
/// `refs/gfs`
pub fn create_gfs_ref(id: impl Display) -> String {
    format!("refs/gfs/{id}")
}

#[cfg(test)]
mod test {
    use crate::create_gfs_ref;

    #[test]
    fn should_create_ref() {
        let id = create_gfs_ref("chair-bro");
        assert_eq!(id, "refs/gfs/chair-bro");
    }
}
